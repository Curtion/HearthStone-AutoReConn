fn plus_one(a: i32) -> (i32, i32) {
    (a, &a + 1)
}

fn main() {
    let (add_num, result) = plus_one(10);
    println!("{} + 1 = {}", add_num, result); // 10 + 1 = 11
}