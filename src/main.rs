#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::io::Read;

lazy_static! {
    static ref TYPEMAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        m.insert("Ptr64", "*mut std::os::raw::c_void");
        m.insert("Uint8B", "u64");
        m.insert("Uint4B", "u32");
        m.insert("Uint2B", "u16");
        m.insert("UChar", "u8");
        m.insert("LARGE_INTEGER", "i64");
        m.insert("_LARGE_INTEGER", "i64");
        m.insert("ULARGE_INTEGER", "i64");
        m.insert("_ULARGE_INTEGER", "i64");
        m.insert("_UNICODE_STRING", "String");
        m.insert("UNICODE_STRING", "String");
        m
    };
}

fn main() -> std::io::Result<()> {
    // create universal buffer to store data in,
    // no matter the method of input.
    let mut buffer = String::new();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        buffer = std::fs::read_to_string(args[1].as_str())
            .expect("Something went wrong reading the file");
    } else {
        std::io::stdin().read_to_string(&mut buffer)?;
    }

    let out = convert_rs(&buffer);

    println!("{}", out);

    Ok(())
}

/// converts the input to a rust formatted struct, ready to use in source files.
fn convert_rs(buffer: &String) -> String {
    let mut lined_buffer = buffer.lines();

    let mut result = String::new();
    {
        // if the input line has been included to us. ie. `kd> dt nt!_PEB`
        // AND the next line showing the module is there (is on newer windbg)
        // we skip it, and use the below version.
        let mut cp_buf = lined_buffer.clone();
        if cp_buf.next().unwrap().contains("!") && cp_buf.next().unwrap().contains("!") {
            lined_buffer.next();
        }
        // Otherwise, we just use the input line!

    }
    result.push_str(
        extract::name_and_module(
            lined_buffer
                .next()
                .expect("Unable to get the first line of input"),
        )
        .as_str(),
    );
    

    // iterate through lines in input buffer, and send to the field processing.
    let mut prev = String::new();
    loop {
        match lined_buffer.next() {
            Some(s) => {
                if s.contains("Pos") {
                    // If this line has same address as line before, we add a comment above it as to note that.
                    if prev != "" {
                        let mut split_line: Vec<&str> = prev.split(' ').collect();
                        split_line.retain(|x| *x != "");
                        let offset = split_line[0];
                        let mut split_current: Vec<&str> = s.split(' ').collect();
                        split_current.retain(|x| *x != "");
                        let splen = split_current.len();
                        let mut offset_cur = split_current[0];
                        offset_cur = &offset_cur[..offset_cur.len()];
                        if offset == offset_cur && prev.contains("Pos") {
                            result.push_str(
                                format!(
                                    "\t// {} at Pos {} {} {} // {}.\n",
                                    split_current[1],
                                    split_current[splen - 3],
                                    split_current[splen - 2],
                                    split_current[splen-1],
                                    split_current[0]
                                )
                                .as_str(),
                            );
                        } else if offset == offset_cur && !prev.contains("Pos") {
                            result.push_str("\t// Bitfield for the above field:\n");
                        }
                    }
                } else {
                    let field_line = extract::field(s);

                    result.push_str(field_line.as_str());
                }
                prev = s.to_string();
            }
            None => {
                result.push_str("}\n");
                break;
            }
        }
    }

    result
}

mod extract {

    pub fn name_and_module(line: &str) -> String {

        let split: Vec<&str> = line.split("dt ").collect();
        let mut module = "";
        let mut name = "";
        if split.len() > 1 {
            let vecc:Vec<&str> = split[1].split('!').collect();
            module = vecc[0];
            name = vecc[1];
        } else {
            let vecc: Vec<&str> = split[0].split('!').collect();
            module = vecc[0];
            name = vecc[1];
        }

        String::from(format!(
            "// {0}!{1}\nstruct {1} {{\n",
            module,
            name
        ))
    }

    pub fn field(line: &str) -> String {

        if line.len() == 0 || line.len() == 1 {
            return String::new();
        }
        let mut out = String::with_capacity(line.len());

        // split the line into:
        // Offset Field Name              Name
        // ------ ---------------------   -----
        // +0x000 InheritedAddressSpace : UChar

        let mut split_line: Vec<&str> = line.split(' ').collect();
        split_line.retain(|x| *x != "");
        
        let offset = split_line[0];


        let mut field_name = split_line[1].to_string();
        field_name.push(':');
        let mut field_type: &str = "";

        if let Some(s) = crate::TYPEMAP.get(split_line[split_line.len() - 1]) {
            field_type = s;
        } else if let Some(s) = crate::TYPEMAP.get(split_line[split_line.len() - 2]) {
            field_type = s;
        } else {
            //println!("COULDNT FIND TYPE FOR: {}", split_line[split_line.len()-1]);
            field_type = split_line[split_line.len() - 1];
        }

        // detect array
        if line.contains('[') {
            let mut num: String = String::new();
            split_line.into_iter().for_each(|i| {
                if !i.contains(']') {
                    return;
                }
                num = format!("{}", &i[1..i.len() - 1]);
            });
            let s = format!("[{}; {}],", field_type, num);
            out.push_str(format!("\t{0:16} {1} // {2}\n", field_name, s, offset).as_str());
        } else {
            let s = format!("{0}, // {1}\n", field_type, offset);
            out.push_str(format!("\t{:16} {}", field_name, s).as_str());
        }

        out
    }
}