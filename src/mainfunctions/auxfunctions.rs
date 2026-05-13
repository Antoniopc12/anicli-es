use std::str;

// Devuelve el código fuente de una página web como String (GET)
pub fn get_source(url: String) -> Result<String, ureq::Error> {
    // Es buena práctica usar un Agent o añadir un header para evitar bloqueos
    let response = ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0")
        .call()?
        .into_string()?;
    Ok(response)
}

// Realiza una petición POST enviando datos en el cuerpo (body)
pub fn post_request(url: String, body: String) -> Result<String, ureq::Error> {
    let response = ureq::post(&url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0")
        .send_string(&body)? // Aquí es donde se envía "value=busqueda"
        .into_string()?;
    Ok(response)
}

// Decodifica un String de base64 a un String
pub fn decode_base64(input: String) -> String {
    let bytes = base64::decode(input).expect("Error al decodificar Base64");
    
    match str::from_utf8(&bytes) {
        Ok(v) => v.to_string(),
        Err(e) => panic!("Secuencia UTF-8 inválida: {}", e),
    }
}