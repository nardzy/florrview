use std::{fs::{self}, io::{Cursor, Write}, path::PathBuf, str::Utf8Error};
use rfd::FileDialog;
use wasmtime::*;
use zip::{ZipWriter, write::SimpleFileOptions};

#[repr(u8)]
#[derive(Copy, Clone)]
enum OpCode {
    Block = 0x02,
    If = 0x04,
    End = 0x0b,
    Br = 0x0c,
    // BrIf = 0x0d,
    Call = 0x10,
    LocalGet = 0x20,
    LocalSet = 0x21,
    I32Const = 0x41,
    I32Store = 0x36,
    Eqz = 0x45,
}

enum Pattern {
    Op(OpCode),
    // OpOr(OpCode, OpCode),
    U8(u8),
    LEB,
}

fn read_leb(bin: &[u8], a: usize, b: usize) -> (usize, i32) {
    let mut result: i32 = 0;
    let mut shift: u8 = 0;
    let mut count = 1;
    for i in a..b {
        let byte = bin[i];
        result |= ((byte & 0x7f) as i32) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        count += 1;
    }
    (count, result)
}

fn read_string(bin: &[u8], p: usize) -> Result<&str, Utf8Error> {
    let mut len = p;
    while bin[len] != 0 {
        len += 1;
    }
    str::from_utf8(&bin[p..len])
}

// ptr 300

pub struct Asset {
    pub path: String,
    pub content: String
}
type DynError = Box<dyn std::error::Error>;
pub fn extract_wasm(path: &PathBuf) -> Result<Vec<Asset>, DynError> {

    let bin = fs::read(path)?;
    let mut config = Config::new();
    config.strategy(Strategy::Winch);
    let engine = Engine::new(&config)?;
    let module = Module::new(&engine, &bin)?;

    let mut mem_name = "";
    for export in module.exports() {
        if let ExternType::Memory(_) = export.ty() {
            mem_name = export.name();
            break;
        }
    }
    if mem_name.is_empty() {
       return Err("why.".into());
    }

    let mut linker = Linker::new(&engine);
    linker.define_unknown_imports_as_traps(&module)?;

    let mut store = Store::new(&engine, 0);

    let instance = linker.instantiate(&mut store, &module)?;
    let memory = instance.get_memory(&mut store, mem_name).ok_or(
        "memory"
    )?;
    let data = memory.data(&mut store);

    let search_func = vec![
        OpCode::LocalGet as u8, 1,
        OpCode::I32Const as u8, 0,
        OpCode::I32Store as u8, 2, 0,
        OpCode::LocalGet as u8, 2,
        OpCode::I32Const as u8, 0,
        OpCode::I32Store as u8, 2, 0,
        OpCode::Block as u8, 64,
        OpCode::Block as u8, 127,
    ];
    let search_func_range = vec![
        OpCode::End as u8,
        OpCode::LocalSet as u8, 0,
        OpCode::LocalGet as u8, 1,
        OpCode::LocalGet as u8, 3,
        OpCode::I32Store as u8, 2, 0,
        OpCode::LocalGet as u8, 2,
        OpCode::LocalGet as u8, 0,
        OpCode::I32Store as u8, 2, 0,
        OpCode::End as u8
    ];
    let mut index = 0;
    let mut start = 0;
    let mut end = 0;

    for (i, &u) in bin.iter().enumerate() {
        if start == 0 {
            if u == search_func[index] {
                if index == search_func.len() - 1 {
                    start = i + 1;
                    index = 0;
                }
                index += 1;
                continue;
            }
            index = 0;
            continue;
        }
        if end == 0 {
            if u == search_func_range[index] {
                if index == search_func_range.len() - 1 {
                    end = i - index;
                    index = 0;
                }
                index += 1;
                continue;
            }
            index = 0;
            continue;
        }
        break;
    }

    if start == 0 || end == 0 {
        return Err("failed to find func. start: {start}, end: {end}".into());
    }

    let mut index = 0;
    let mut at = start;

    let patterns = vec![
        Pattern::Op(OpCode::LocalGet), Pattern::U8(0),
        Pattern::Op(OpCode::I32Const), Pattern::LEB,
        Pattern::Op(OpCode::Call), Pattern::LEB, 
        Pattern::Op(OpCode::Eqz),
        Pattern::Op(OpCode::If), Pattern::U8(64), // 64
        Pattern::Op(OpCode::I32Const), Pattern::LEB,
        Pattern::Op(OpCode::LocalSet), Pattern::U8(3),
        Pattern::Op(OpCode::I32Const), Pattern::LEB,
        Pattern::Op(OpCode::Br), Pattern::U8(1),
        Pattern::Op(OpCode::End) 
    ];
    // let skip = 3;

    let mut args = [0_u32; 64];
    let mut arg_i = 0;
    let mut assets = Vec::new();
    while at < end {
        // if bin[at] == OpCode::BrIf as u8 {
        //     println!("{:?}", &bin[at..(at + 32)]);
        // }
        match &patterns[index] {
            Pattern::Op(op) => {
                let op = *op as u8;
                if op != bin[at] {
                    break;
                }
                at += 1;
            }
            // Pattern::OpOr(op1, op2) => {
            //     let op1 = *op1 as u8;
            //     let op2 = *op2 as u8;
            //     if op1 != bin[at] && op2 != bin[at] {
            //         break;
            //     }
            //     at += 1;
            // }
            Pattern::U8(u) => {
                if *u != bin[at] {
                    break;
                }
                at += 1;
            },
            Pattern::LEB => {
                let (
                    count,
                    value
                ) = read_leb(&bin, at, at + 4);
                args[arg_i] = value as u32;
                arg_i += 1;
                at += count;
            },
        }
        if index == patterns.len() - 1 {
            // at += skip;
            index = 0;
            arg_i = 0;

            let path = args[0] as usize;
            let content = args[2] as usize; // 3

            let path = read_string(&data, path)?.to_string();
            let content = read_string(&data, content)?.to_string();

            assets.push(
                Asset { path, content }
            );

            // retuyrnnn
            continue;
        }
        index += 1;
    }

    // println!("af {:?}", assets.last().unwrap().path);

    Ok(assets)
}

pub fn compress_to_zip(assets: &Vec<Asset>) -> Result<(), DynError> {

    let mut buf = Vec::new();
    let cursor = Cursor::new(&mut buf);

    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for asset in assets {
        let path = format!("static/{}", asset.path);
        //let Some(dir) = Path::new(&path).parent() else { continue; };
        zip.start_file(path, options)?;
        zip.write_all(asset.content.as_bytes())?;
    }
    let cursor = zip.finish()?;

    let out_path = FileDialog::new()
        .set_title("Save")
        .set_file_name("static.zip")
        .save_file();

    if let Some(path) = out_path {
        let res = cursor.into_inner();
        fs::write(path, res)?;
    }


    Ok(())

}