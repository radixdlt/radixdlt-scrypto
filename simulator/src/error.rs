pub fn exit_with_error(msg: String, exit_code: i32) {
    eprintln!("{}", msg);
    std::process::exit(exit_code)
}
