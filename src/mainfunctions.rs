use dialoguer::Input;
use regex::Regex;
use std::ops::Index;
use std::process::Command;
use serde_json::Value; // Importación necesaria para el nuevo parseo

mod auxfunctions;

// Recibe un largo(usize) en el cual elegir un índice y un String que dice a que corresponde el
// índice a elegir y entrega el input del usuario como entero i32
pub fn choose_index(lenght: usize, que: &str) -> i32 {
    if lenght == 1 {
        return 0;
    }
    loop {
        let mut prompt: String = "Elige un ".to_string();
        prompt.push_str(que);

        let index: String = Input::new().with_prompt(prompt).interact().unwrap();

        let index: i32 = match index.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Porfavor escribe un número.");
                continue;
            }
        };

        if index > lenght.try_into().unwrap() {
            println!("El índice seleccionado es invalido.");
            continue;
        } else if index < 1 {
            println!("El índice seleccionado es invalido.");
            continue;
        } else {
            return index - 1;
        }
    }
}

// Búsqueda mediante la API de AnimeFLV usando POST
pub fn search_query(query: String) -> String {
    let url = "https://www4.animeflv.net/api/animes/search".to_string();
    let body = format!("value={}", query);

    // Se envía el body como segundo argumento
    match auxfunctions::post_request(url, body) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error en la búsqueda: {}", e);
            String::new()
        }
    }
}

// Procesa el JSON de respuesta de AnimeFLV
pub fn query_results(source: String) -> Vec<Vec<String>> {
    let json: Value = serde_json::from_str(&source).unwrap_or(Value::Null);
    
    let mut titles = Vec::new();
    let mut links = Vec::new();
    let mut categories = Vec::new();

    if let Some(array) = json.as_array() {
        for item in array {
            let title = item["title"].as_str().unwrap_or("").to_string();
            let slug = item["slug"].as_str().unwrap_or("").to_string();
            
            titles.push(title);
            // Construcción del link para AnimeFLV
            links.push(format!("https://www4.animeflv.net/anime/{}", slug));
            categories.push("Anime".to_string());
        }
    }

    vec![titles, categories, links]
}

pub fn choose_anime(animelist: &Vec<Vec<String>>) -> i32 {
    if animelist[0].is_empty() {
        println!("No se encontraron resultados.");
        return -1;
    }
    
    if animelist[0].len() == 1 {
        return 0;
    } else {
        for n in 0..animelist[0].len() {
            println!("[{}] {} - {}", n + 1, animelist[0][n], animelist[1][n]);
        }
    }

    let animes: String = format!("anime [1-{}]", animelist[0].len());
    choose_index(animelist[0].len(), animes.as_str())
}

pub fn get_episodes(url: String) -> Vec<String> {
    // 1. Obtenemos el código fuente de la página de la ficha (ej: .../anime/one-piece)
    let source = auxfunctions::get_source(url.clone()).expect("No se pudo obtener la fuente");

    // 2. Extraemos el ID interno del anime que usa AnimeFLV para los episodios.
    // A veces el slug de la URL y el ID interno son diferentes, por eso lo buscamos en el HTML.
    let re_info = Regex::new(r"var anime_info = \[(.*)\];").unwrap();
    let re_episodes = Regex::new(r"var episodes = \[(.*)\];").unwrap();

    let mut links: Vec<String> = Vec::new();

    if let (Some(cap_info), Some(cap_eps)) = (re_info.captures(&source), re_episodes.captures(&source)) {
        // El tercer elemento de anime_info suele ser el "id" para las URLs de los episodios
        let info_str = format!("[{}]", cap_info.get(1).unwrap().as_str());
        let info_json: Value = serde_json::from_str(&info_str).unwrap();
        let anime_id = info_json[2].as_str().unwrap_or("");

        // Parseamos la lista de episodios
        let eps_str = format!("[{}]", cap_eps.get(1).unwrap().as_str());
        let eps_json: Value = serde_json::from_str(&eps_str).unwrap();

        if let Some(ep_list) = eps_json.as_array() {
            // AnimeFLV pone los episodios del más nuevo al más viejo, usamos .rev() para ordenarlos
            for ep in ep_list.iter().rev() {
                let num_episodio = ep[0].as_i64().unwrap_or(0);
                
                // IMPORTANTE: La URL de ver episodio usa "/ver/" y el ID del anime
                let link_episodio = format!("https://www4.animeflv.net/ver/{}-{}", anime_id, num_episodio);
                links.push(link_episodio);
            }
        }
    }

    links
} 

// Modificamos el retorno para incluir los nombres de los servidores
pub fn episode_link_scrapper(url: String) -> (Vec<Vec<String>>, Vec<String>) {
    println!("{esc}[2J{esc}[1;1HObteniendo servidores...", esc = 27 as char);

    let video_source = auxfunctions::get_source(url).expect("Error al obtener la página");
    let re_videos = Regex::new(r"var videos = (\{.*\});").unwrap();

    let mut all_links = Vec::new();
    let mut server_names = Vec::new();

    if let Some(caps) = re_videos.captures(&video_source) {
        let json_str = caps.get(1).unwrap().as_str();
        let v: serde_json::Value = serde_json::from_str(json_str).unwrap_or(serde_json::Value::Null);

        if let Some(sub_servers) = v["SUB"].as_array() {
            for server in sub_servers {
                let name = server["server"].as_str().unwrap_or("unknown");
                let code = server["code"].as_str().unwrap_or("");

                // Guardamos el nombre para el menú
                server_names.push(name.to_uppercase());

                // Guardamos el link con su formato para getargs
                if name == "stape" || name == "sw" || name == "streamwish" {
                    all_links.push(vec!["1".to_string(), "https://www4.animeflv.net/".to_string(), code.to_string()]);
                } else {
                    all_links.push(vec!["0".to_string(), code.to_string()]);
                }
            }
        }
    }
    (all_links, server_names)
}

fn getargs(links: Vec<Vec<String>>) -> String {
    for l in links {
        // User agent estándar y simple
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";
        
        if l[0] == "0" && l.len() >= 2 {
            return format!("mpv --ontop --geometry=50% --user-agent=\"{}\" \"{}\"", ua, l[1]);
        } 
        else if l[0] == "1" && l.len() >= 3 {
            // Pasamos el Referer de la forma más compatible posible
            return format!(
                "mpv --ontop --geometry=50% --referrer=\"{}\" --user-agent=\"{}\" \"{}\"", 
                l[1], ua, l[2]
            );
        }
    }
    String::new()
}

pub fn mpv(nombre: &String, links: &[String], episodio: i32) {
    let video_source = auxfunctions::get_source(links[episodio as usize].clone()).expect("Error");
    let re_videos = Regex::new(r"var videos = (\{.*?\});").unwrap();

    let mut urls_servidores = Vec::new();
    let mut nombres_menu = Vec::new();

    if let Some(caps) = re_videos.captures(&video_source) {
        let json_str = caps.get(1).unwrap().as_str();
        if let Ok(v) = serde_json::from_str::<Value>(json_str) {
            if let Some(sub_list) = v["SUB"].as_array() {
                for s in sub_list {
                    let name = s["server"].as_str().unwrap_or("").to_uppercase();
                    let mut code = s["code"].as_str().unwrap_or("").to_string();

                    // Limpieza rápida de la URL para que el navegador la entienda
                    code = code.replace(r"\/", "/");

                    nombres_menu.push(name);
                    urls_servidores.push(code);
                }
            }
        }
    }

    if nombres_menu.is_empty() { return; }

    println!("{esc}[2J{esc}[1;1H--- Abrir en Navegador ---", esc = 27 as char);
    for (i, n) in nombres_menu.iter().enumerate() {
        println!("[{}] {}", i + 1, n);
    }

    let sel = choose_index(nombres_menu.len(), "servidor");
    let url_final = &urls_servidores[sel as usize];

    println!("\nAbriendo {} en tu navegador...", nombres_menu[sel as usize]);

    // Comando universal para abrir el navegador predeterminado en Linux
    let _ = Command::new("xdg-open")
        .arg(url_final)
        .spawn();

    // Volvemos al control de episodios para que puedas saltar al siguiente tras abrirlo
    controller(episodio, links.to_vec(), nombre);
}

fn controller(index: i32, links: Vec<String>, nombre: &String) {
    let mut case: i8 = 0;
    let linkslen = links.len();
    
    // CORRECCIÓN E0283: Se define el tipo explícitamente para evitar ambigüedad con serde_json
    let len_i32: i32 = linkslen.try_into().unwrap();

    loop {
        let mut prompt = String::from("");

        if index == 0 && index + 1 == len_i32 {
            // Solo un episodio disponible
        } else if index == 0 {
            prompt.push_str("[s] Siguiente episodio\n");
            case = 1;
        } else if index + 1 == len_i32 {
            prompt.push_str("[a] Anterior episodio\n");
            case = 2;
        } else {
            prompt.push_str("[a] Anterior episodio\n[s] Siguiente Episodio\n");
            case = 3;
        }

        prompt.push_str("[r] Ver de nuevo\n[o] Seleccionar otro episodio\n[b] Buscar otro anime\n[q] Salir\nEscoge una opción");

        let opcion: String = Input::new().with_prompt(prompt).interact().unwrap();
        let opcion = opcion.trim().to_lowercase();

        match opcion.as_str() {
            "q" => std::process::exit(0),
            "b" => break,
            "r" => mpv(nombre, &links, index),
            "o" => {
                let nuevo_idx = choose_index(links.len(), format!("episodio [1-{}]", links.len()).as_str());
                mpv(nombre, &links, nuevo_idx);
            },
            "s" if case == 1 || case == 3 => mpv(nombre, &links, index + 1),
            "a" if case == 2 || case == 3 => mpv(nombre, &links, index - 1),
            _ => {
                println!("Escoge una opción valida.\n");
                continue;
            }
        }
    }
}