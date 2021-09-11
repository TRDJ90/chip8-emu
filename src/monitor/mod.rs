

pub mod monitor {
    pub const WIDTH: usize = 64;
    pub const HEIGHT: usize = 32;

    #[derive(Debug)]
    pub struct Monitor {
        pub pixels: [bool; 2048]
    }

    impl Monitor {
        pub fn new() -> Monitor {
            Monitor {
                pixels: [false; WIDTH * HEIGHT]
            }
        }

        pub fn set_pixel(&mut self, x: usize, y: usize) {
            let index = x + (WIDTH * y);
            self.pixels[index] ^= true;
        }

        pub fn clear(&mut self) {
            self.pixels = [false; WIDTH * HEIGHT]
        } 
    }
}