mod package_loader;
mod common;

mod application {
    automod::dir!("tests/application");
}

mod blueprints {
    automod::dir!("tests/blueprints");
}

mod system {
    automod::dir!("tests/system");
}

mod vm {
    automod::dir!("tests/vm");
}

mod kernel {
    automod::dir!("tests/kernel");
}

mod db {
    automod::dir!("tests/db");
}
