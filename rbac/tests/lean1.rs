use anyhow::anyhow;

#[test]
fn match_test() {
    match find_user(2) {
        Some(name) => {
            println!("name:{}", name)
        }
        None => {
            println!("Not found")
        }
    }
}

#[test]
fn if_let_test() {
    if let Some(name) = find_user(1) {
        println!("name:{}", name)
    }
}

#[test]
fn unwrap_test() {
    let name = find_user(11).unwrap();
    println!("name:{}", name)
}

#[test]
fn except_test() {
    find_user(123).expect("user should exist");
}

#[test]
fn unwrap_or_test() {
    let name = find_user(123).unwrap_or("Unknow".to_string());
    println!("name:{}", name)
}

#[test]
fn unwrap_or_else_test() {
    let name = find_user(123).unwrap_or_else(|| "Unknow".to_string());
    println!("name:{}", name)
}

#[test]
fn option_map() {
    let name = find_user(1).map(|name| name.len());
    println!("name len:{:?}", name)
}

#[test]
fn option_and_then() {
    // let age = Some("18".to_string()).and_then(parse);
    let age = Some("18".to_string()).and_then(parse);
    println!("age:{:?}", age);
    let age = Some("18".to_string()).map(|age| age.parse::<u32>().ok());
    println!("age:{:?}", age);
}

#[test]
fn option_ok_or() {
    let user = find_user(2).ok_or(anyhow!("user not found"));
    print!("user:{:?}", user);
}



fn parse(s: String) -> Option<u32> {
    s.parse().ok()
}

fn find_user(id: i64) -> Option<String> {
    if id == 1 {
        Some("Tom".to_string())
    } else {
        None
    }
}
