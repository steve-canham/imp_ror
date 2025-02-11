use imp_ror::err;
use imp_ror::run;
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() {

    let args: Vec<_> = env::args_os().collect();
    match run(args).await
    {
        Ok(_) => println!("Done!"),
        Err(e) => err::report_error(e),
    };
}