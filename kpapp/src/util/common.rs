use nanoid::nanoid;
pub fn generate_unique_string() -> String {
    nanoid!(8)
}


#[cfg(test)]
mod tests {
    use log::info;
    use crate::init::initialize;
    use super::*;

    #[test]
    fn test_generate_unique_string() {
        initialize();
        let unique_string1 = generate_unique_string();
        let unique_string2 = generate_unique_string();

        info!("{}", unique_string1);
        info!("{}", unique_string2);
    }
}