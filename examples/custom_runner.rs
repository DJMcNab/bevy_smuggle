use bevy_app::prelude::*;
use bevy_smuggle::RefResMut;

fn main() {
    App::new()
        .add_system(do_something_with_ten)
        .set_runner(|app| {
            let mut world = app.world;
            let mut context = MyContext;
            // In the 2021 edition, rfc 2229 will make this manual extraction not needed
            let mut schedule = app.schedule;
            bevy_smuggle::temporarily_store_exclusive_ref(&mut world, &mut context, |world| {
                schedule.run_once(world)
            })
        })
        .run();
}

fn do_something_with_ten(mut ctx: RefResMut<MyContext>) {
    ctx.do_something(10);
}

struct MyContext;

impl MyContext {
    fn do_something(&mut self, value: u32) {
        println!("The value is {}", value)
    }
}
