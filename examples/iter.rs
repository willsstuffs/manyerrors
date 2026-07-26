use manyerrors::{Errors, manyerrors, iter_stash, LazyErrs};

fn main() {
    println!("{:#?}", my_fn());
}

#[manyerrors(String)]
fn my_fn() -> Result<(),Errors<String>> {
    assert_eq!(
        iter_stash!(vec![Ok(1),Ok(2),Err(String::from("error")),Ok(3)]).collect::<Vec<_>>(),
        vec![1,2,3]
    );
    assert_eq!(
        [Ok(1),Err(1),Ok(2),Err(2),Ok(3),Err(3)].into_iter().lazy_errs().collect::<Vec<_>>(),
        vec![Ok(1),Ok(2),Ok(3),Err(Errors::<i32>::from(vec![1,2,3]))]
    );
    Ok(())
}