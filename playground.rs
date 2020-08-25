struct Aparte {
    closure: Option<Box<dyn FnMut(&'static mut Aparte)>>,
}

impl Aparte {
    fn setclosure<F: 'static + FnMut(&'static mut Aparte)>(&'static mut self, closure: F) {
        self.closure = Some(Box::new(closure));
    }
    fn func(&'static mut self) {
        println!("func");
    }

    fn register<F: FnMut()>(&'static mut self, callback: F) {
        callback();
    }
}

fn main() {
    let mut aparte = Aparte {
        closure: None,
    };

    aparte.setclosure(move |aparte| {
        aparte.register(move || {
            aparte.func();
        });
    });
}
