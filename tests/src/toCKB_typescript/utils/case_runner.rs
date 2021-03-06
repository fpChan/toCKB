use super::case_builder::{
    CellBuilder, OutpointsContext, TestCase, ALWAYS_SUCCESS_OUTPOINT_KEY, FIRST_INPUT_OUTPOINT_KEY,
    SUDT_TYPESCRIPT_OUTPOINT_KEY, TOCKB_LOCKSCRIPT_OUTPOINT_KEY, TOCKB_TYPESCRIPT_OUTPOINT_KEY,
};
use crate::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{
    core::TransactionBuilder,
    packed::{CellDep, CellInput, CellOutput},
    prelude::*,
};
use std::mem::replace;

pub const MAX_CYCLES: u64 = 100_000_000;

pub fn run_test(case: TestCase) {
    let mut context = Context::default();
    let mut outpoints_context = OutpointsContext::new();

    // Cell deps
    let mut cell_deps = vec![];
    // Custom cell deps
    for cell_dep_view in case.cell_deps.iter() {
        cell_deps.push(cell_dep_view.build_cell_dep(&mut context));
    }
    // Script cell deps
    deploy_scripts(&mut context, &mut outpoints_context);
    for (_, v) in outpoints_context.iter() {
        let cell_dep = CellDep::new_builder().out_point(v.clone()).build();
        cell_deps.push(cell_dep);
    }

    // Cells
    let inputs_len = case.toCKB_cells.inputs.len()
        + case.sudt_cells.inputs.len()
        + case.capacity_cells.inputs.len();
    let outputs_len = case.toCKB_cells.outputs.len()
        + case.sudt_cells.outputs.len()
        + case.capacity_cells.outputs.len();
    let mut inputs = vec![CellInput::default(); inputs_len];
    let mut outputs = vec![CellOutput::default(); outputs_len];
    let mut outputs_data = vec![Bytes::default(); outputs_len];

    build_input_cell(
        case.toCKB_cells.inputs.into_iter(),
        &mut context,
        &mut outpoints_context,
        &mut inputs,
    );
    build_input_cell(
        case.sudt_cells.inputs.into_iter(),
        &mut context,
        &mut outpoints_context,
        &mut inputs,
    );
    build_input_cell(
        case.capacity_cells.inputs.into_iter(),
        &mut context,
        &mut outpoints_context,
        &mut inputs,
    );

    build_output_cell(
        case.toCKB_cells.outputs.into_iter(),
        &mut context,
        &mut outpoints_context,
        &mut outputs,
        &mut outputs_data,
    );
    build_output_cell(
        case.sudt_cells.outputs.into_iter(),
        &mut context,
        &mut outpoints_context,
        &mut outputs,
        &mut outputs_data,
    );
    build_output_cell(
        case.capacity_cells.outputs.into_iter(),
        &mut context,
        &mut outpoints_context,
        &mut outputs,
        &mut outputs_data,
    );

    dbg!("inputs: {:?}", &inputs);
    dbg!("outputs: {:?}", &outputs);

    // Witnesses
    let mut witnesses = vec![];
    for witness in case.witnesses {
        witnesses.push(witness.as_bytes().pack());
    }

    // Build tx
    let tx = TransactionBuilder::default()
        .cell_deps(cell_deps)
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .witnesses(witnesses)
        .build();
    let tx = context.complete_tx(tx);
    dbg!(&tx);

    // Test tx
    let res = context.verify_tx(&tx, MAX_CYCLES);
    dbg!(&res);
    match res {
        Ok(_cycles) => assert_eq!(case.expect_return_code, 0),
        Err(err) => assert!(check_err(err, case.expect_return_code)),
    }
}

fn deploy_scripts(context: &mut Context, outpoints_context: &mut OutpointsContext) {
    let toCKB_typescript_bin: Bytes = Loader::default().load_binary("toCKB-typescript");
    let toCKB_typescript_out_point = context.deploy_cell(toCKB_typescript_bin);
    let sudt_typescript_bin = include_bytes!("../../../deps/simple_udt");
    let sudt_typescript_out_point = context.deploy_cell(Bytes::from(sudt_typescript_bin.as_ref()));
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    outpoints_context.insert(
        TOCKB_TYPESCRIPT_OUTPOINT_KEY,
        toCKB_typescript_out_point.clone(),
    );
    outpoints_context.insert(
        TOCKB_LOCKSCRIPT_OUTPOINT_KEY,
        always_success_out_point.clone(),
    );
    outpoints_context.insert(
        SUDT_TYPESCRIPT_OUTPOINT_KEY,
        sudt_typescript_out_point.clone(),
    );
    outpoints_context.insert(
        ALWAYS_SUCCESS_OUTPOINT_KEY,
        always_success_out_point.clone(),
    );
}

fn build_input_cell<I, B>(
    iterator: I,
    context: &mut Context,
    outpoints_context: &mut OutpointsContext,
    inputs: &mut Vec<CellInput>,
) where
    I: Iterator<Item = B>,
    B: CellBuilder,
{
    for input in iterator {
        let index = input.get_index();
        let (input_outpoint, input_cell) = input.build_input_cell(context, outpoints_context);
        let _old_value = replace(&mut inputs[index], input_cell);
        if 0 == index {
            outpoints_context.insert(FIRST_INPUT_OUTPOINT_KEY, input_outpoint);
        }
    }
}

fn build_output_cell<I, B>(
    iterator: I,
    context: &mut Context,
    outpoints_context: &mut OutpointsContext,
    outputs: &mut Vec<CellOutput>,
    outputs_data: &mut Vec<Bytes>,
) where
    I: Iterator<Item = B>,
    B: CellBuilder,
{
    for output in iterator {
        let index = output.get_index();
        let (output_data, output_cell) = output.build_output_cell(context, outpoints_context);
        let _old_value = replace(&mut outputs[index], output_cell);
        let _old_value = replace(&mut outputs_data[index], output_data);
    }
}

fn check_err(err: ckb_tool::ckb_error::Error, code: i8) -> bool {
    let get = format!("{}", err);
    let expected = format!("Script(ValidationFailure({}))", code);
    dbg!(&get, &expected);
    get == expected
}
