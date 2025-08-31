fn main() {
    let f1 = 10.50_f64;
    let f2 = 20.00_f64;
    let f3 = 15.75_f64;
    let f4 = 100_i64;

    println!("10.50 -> {}", f1);
    println!("20.00 -> {}", f2);
    println!("15.75 -> {}", f3);
    println!("100 (int) -> {}", f4);

    println!("\nContains '.':");
    println!("10.50 contains '.': {}", f1.to_string().contains('.'));
    println!("20.00 contains '.': {}", f2.to_string().contains('.'));
    println!("15.75 contains '.': {}", f3.to_string().contains('.'));
    println!("100 contains '.': {}", f4.to_string().contains('.'));
}
