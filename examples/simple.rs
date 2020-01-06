fn main() {
    let x = (0..300).into_iter().map(|_| 0u8).collect::<Vec<u8>>();

    let buf =
        split_buffer::Buffer::build(&[x.as_slice(), &[2, 3, 4][..], &[5, 5, 5][..], &[1, 1][..]]);

    println!("{:?}", &buf);

    for part in &buf {
        println!("{:?}", &part);
    }
}
