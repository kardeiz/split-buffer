fn main() {
    let buf = spliff::Config::default()
        .build(&[&[1, 3][..], &[2, 3, 4][..], &[5, 5, 5][..], &[1, 1][..]])
        .unwrap();

    println!("{:?}", &buf);

    for part in &buf {
        println!("{:?}", &part);
    }
}
