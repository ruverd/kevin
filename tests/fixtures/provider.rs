use std::io::{Read,Write};
fn quoted(s:&str)->String {
    let mut out=String::from("\"");for c in s.chars(){match c {'"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),'\n'=>out.push_str("\\n"),'\r'=>out.push_str("\\r"),'\t'=>out.push_str("\\t"),_=>out.push(c)}}out.push('"');out
}
fn main() {
    let args:Vec<String>=std::env::args().collect();
    let mode=args.get(1).map(String::as_str).unwrap_or("draft");
    let mut input=String::new();std::io::stdin().read_to_string(&mut input).unwrap();
    match mode {
        "invalid"=>print!("not-json"),
        "failure"=>{eprint!("PRIVATE-TOKEN");std::process::exit(3);},
        "huge"=>{let _=std::io::stdout().write_all(&vec![b'x';1_000_001]);},
        "sleep"=>std::thread::sleep(std::time::Duration::from_secs(2)),
        "echo"=>print!("{{\"status\":\"draft\",\"text\":{}}}",quoted(&args[2])),
        "review"=>print!("{{\"status\":\"draft\",\"text\":\"Guard items[0].id first. Tests have not run.\"}}"),
        _=>print!("{{\"status\":\"draft\",\"text\":\"Coffee still needs a human in the loop.\"}}"),
    }
}
