#![no_std]
#![no_main]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![allow(non_snake_case)]

mod utils;

use core::result::Result;
use ckb_std::{
    ckb_types::{bytes::Bytes, prelude::*},
    debug, default_alloc, entry,
    error::SysError,
    high_level::{load_script_hash,load_cell_type_hash,load_cell,load_cell_lock_hash},
    ckb_constants::Source,
};
use utils::error::Error;
use alloc::{vec};
use hex;
use ckb_std::high_level::load_script;
entry!(entry);
default_alloc!();

/// Program entry
fn entry() -> i8 {
    // Call main function and return error code
    match main() {
        Ok(_) => 0,
        Err(err) => err as i8,
    }
}

fn main() -> Result<(), Error> {
    verify()
}


fn verify() -> Result<(), Error> {
    // load current lock_script hash
    let script_hash = load_script_hash().unwrap();
    let args: Bytes  = load_script().unwrap().args().unpack();
    let cell_source = vec![Source::Input,Source::Output];

    for &source in cell_source.iter() {
        for i in 0.. {
            match verify_single_cell(i, source, script_hash, args.clone()) {
                Ok(()) => {}
                Err(Error::IndexOutOfBound) => break,
                Err(err) => return Err(err.into()),
            };
        }
    }
    Ok(())
}

fn verify_single_cell(index: usize, source: Source, script_hash: [u8; 32], args:Bytes) -> Result<(), Error> {

    let type_hash = match load_cell_type_hash(index, source) {
        Ok(current_cell_type_hash) => current_cell_type_hash,
        Err(SysError::IndexOutOfBound) => return Err(Error::IndexOutOfBound),
        Err(err) => return Err(err.into()),
    };

    debug!("test_lock_args : {:?}-{:?} \t lock script args : {:?} \t type script hash : {:?} \n",
           source, index, hex::encode(&args.to_vec()), hex::encode(type_hash.clone().unwrap()));

    // the cell is toCKBCell when lock_script args equal typescript hash
    if args[..] == type_hash.clone().unwrap()[..] {

        let lock_hash = match load_cell_lock_hash(index, source) {
            Ok(lock_hash) => lock_hash,
            Err(SysError::IndexOutOfBound) => return Err(Error::IndexOutOfBound),
            Err(err) => return Err(err.into()),
        };

        debug!("test_lock_hash : {:?}-{:?} \t lock_script hash : {:?} \t current lock_hash : {:?}\n \n",
               source, index, hex::encode(lock_hash.clone()), hex::encode(script_hash.clone()));

        //the toCKBCell is valid when the toCKB cell lock_script hash equal current lock_script hash
        if lock_hash[..] != script_hash[..] {
            return Err(Error::InvalidToCKBCell);
        }
    }
    Ok(())
}