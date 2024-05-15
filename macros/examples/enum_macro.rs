use macros::EnumFrom;

#[allow(unused)]
#[derive(Debug, EnumFrom)]
enum Direction<T> {
    Up(DirectionUp<T>),
    Down,
    Left(u32),
    Right { a: u32 },
}

#[allow(unused)]
#[derive(Debug)]
struct DirectionUp<T> {
    speed: T,
}

fn main() {
    let up: Direction<i32> = DirectionUp::new(42).into();
    let left: Direction<i32> = 10.into();
    println!("Up: {:?}, Left: {:?}", up, left);
}

impl<T> DirectionUp<T> {
    fn new(speed: T) -> Self {
        Self { speed }
    }
}

// ident: Direction, var: Up, ty: DirectionUp
// impl From<DirectionUp> for Direction {
//     fn from(v: DirectionUp) -> Self {
//         Direction::Up(v)
//     }
// }