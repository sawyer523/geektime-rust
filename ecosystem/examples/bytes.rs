use anyhow::Result;
use bytes::{BufMut, BytesMut};

fn main() -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);
    buf.extend_from_slice(b"hello world\n");
    buf.put(&b"goodbye world\n"[..]);
    buf.put_i64(0xdeadbeef);

    println!("{:?}", buf);
    let a = buf.split();
    println!("{:?}", a);
    println!("{:?}", buf);
    Ok(())
}
