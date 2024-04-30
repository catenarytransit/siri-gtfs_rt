pub fn string_h_m_to_u32(x: String) -> u32 {
    //split string into h and m based on :

    let mut input = x
        .clone()
        .replace("!", "")
        .replace("#", "")
        .replace("*", "")
        .replace("$", "")
        .replace(" ", "");

    //it's okay if it runs over 24, that's expected
    let mut h = input.split(":").nth(0).unwrap().parse::<u32>().unwrap();
    let mut m = input.split(":").nth(1).unwrap().parse::<u32>().unwrap();

    (h * 3600) + (m * 60)
}
