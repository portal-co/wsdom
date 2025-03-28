fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next();
    let p = args.next().unwrap();
    return std::fs::write(p, px_wsdom_gen::gen::<&str>(&[],&Default::default()));
}
