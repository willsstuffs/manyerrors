use manyerrors::{Errors, err, manyerrors, stash};

fn main() {
    println!("{:#?}",my_fn());
}

#[manyerrors(String)]
fn my_fn() -> Result<(),Errors<String>> {
    stash!(err(String::from("error")));
    stash!(err(String::from("error2")));
    stash!(err(String::from("error3")),(Ok(()) as Result<_,String>));
    Err(String::from("Short circuiting"))?;
    stash!(err(String::from("error4")));
    Ok(())
}