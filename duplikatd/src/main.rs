mod backups;
mod restic;

use backups as backups_routes;
use restic as restic_routes;

mod index {
    vial::routes! {
        GET "/" => |_| "<h1>This is the index.</h1>";
    }
}

fn main() {
    vial::run!(
        index,
        backups_routes,
        restic_routes
    ).unwrap()
}
