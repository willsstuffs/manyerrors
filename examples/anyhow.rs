use manyerrors::anyhow::Result;
use manyerrors::manyerrors;
use anyhow::anyhow;
use manyerrors::err;
use manyerrors::stash;

pub struct Acc {
    v: f32
}

impl Acc {
    #[manyerrors]
    fn new(v: f32) -> Result<Acc> {
        Ok(
            Acc { v: v }
        )
    }

    #[manyerrors]
    fn increment(&mut self) -> Result<()> {
        self.assert_not_nan()?; //short circuiting, cant increment nan
        self.v += 1.0f32;
        Ok(())
    }

    #[manyerrors]
    fn invert(&mut self) -> Result<()> {
        //not short circuiting, waits until fn exit to return errors
        let o = stash!(self.reciprocal());
        if let Some(v) = o {
            self.v = v;
        } else {
            self.v = f32::NAN;
        }
        Ok(())
    }

    #[manyerrors]
    fn assert_not_nan(&self) -> Result<()> {
        if self.v.is_nan() {
            Err(err(anyhow!("Cant do arithmetic with a nan value!")))
        } else {
            Ok(())
        }
    }

    #[manyerrors]
    fn reciprocal(&self) -> Result<f32> {
        self.assert_not_nan()?; //short circuiting, cant devide by nan
        if self.v == 0.0f32 {
            return Err(err(anyhow!("cannot divide by zero!")));
        }
        Ok(1.0f32/self.v)
    }
}

#[manyerrors]
fn myfunc(inp: f32) -> Result<()> {
    let mut value = Acc::new(inp)?;
    stash!(value.invert());
    stash!(value.increment());
    println!("Value of 1/{}+1 is {}", inp, value.v);
    Ok(())
}

#[manyerrors]
fn main() -> Result<()> {
    stash!(myfunc(1.0f32));
    stash!(myfunc(0.0f32));
    stash!(myfunc(2.0f32));
    Ok(())
}