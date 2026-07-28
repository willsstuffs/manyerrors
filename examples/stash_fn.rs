use manyerrors::{stash_fn, Errors, manyerrors};

fn div2(inp: i32) -> Result<i32,String> {
    if inp % 2 != 0 {
        return Err(format!("{inp} is not an even number!"));
    }
    Ok(inp / 2)
}

#[manyerrors(String)]
fn main() -> Result<(),Errors<String>> {
    for i in 0..=10 {
        stash_fn! {
            println!("{i} / 2 = {}",div2(i)?)
        };
    }
    Ok(())
}