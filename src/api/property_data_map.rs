use serde::Deserialize;
use std::collections::HashMap;

fn get_form_data(suburb_code: &str) -> HashMap<&str, &str> {
    HashMap::from([
        ("Local_Language", "ZHS"),
        ("Local_Country", "AUS"),
        ("Local_State", "NSW"),
        ("Local_Suburb", suburb_code),
        ("Menu_Lv1", "Utilities"),
        ("Menu_Lv2", "Electricity Line"),
        ("CurrentLocation_Lat", ""),
        ("CurrentLocation_Lon", ""),
    ])
}

pub fn server_init_init() -> HashMap<String, [String; 4]> {
    #[derive(Deserialize, Debug)]
    struct RawInitResponse {
        #[serde(rename(deserialize = "Array_Suburb"))]
        array_suburb: String,
    }

    let client = reqwest::blocking::Client::new();
    let endpoint =
        "https://www.propertydatamap.com.au/Property/00_PHP_9/Server_Initial_Initial.php";
    let response = client
        .post(endpoint)
        .form(&get_form_data("4167"))
        .send()
        .unwrap()
        .json::<RawInitResponse>()
        .unwrap();
    let raw_suburb_map =
        serde_json::from_str::<HashMap<String, [String; 4]>>(&response.array_suburb)
            .expect("failed to parse ARRAY_SUBURB to map[str -> [str; 4]]");
    trace!("suburb post code raw data received");
    raw_suburb_map
}

#[derive(Deserialize, Debug)]
pub struct SuburbPolygon {
    pub r#type: String,
    pub coordinates: Vec<Vec<[f64; 2]>>,
}

#[derive(Deserialize, Debug)]
pub struct SelectedLatLon {
    pub r#type: String,
    pub coordinates: Vec<[f64; 3]>,
}

pub struct SelectSuburbResponse {
    pub selected_lat_lon: HashMap<String, SelectedLatLon>,
    pub selected_popup_info: HashMap<String, Vec<String>>,
}

pub fn select_suburb(suburb_id: u16, suburb_name: &str) -> SelectSuburbResponse {
    #[derive(Deserialize, Debug)]
    struct RawSelectSuburbResponse {
        #[serde(rename(deserialize = "Array_Data"))]
        array_data: String,
    }

    #[derive(Deserialize, Debug)]
    struct RawArrayData {
        #[serde(rename(deserialize = "Geometry_Selected_LatLon"))]
        geometry_selected_latlon: HashMap<String, String>,

        #[serde(rename(deserialize = "Geometry_Selected_Popup_Info"))]
        geometry_selected_popup_info: HashMap<String, Vec<String>>,
    }

    debug!("fetching suburb response parsed for {}", suburb_name);
    let client = reqwest::blocking::Client::new();
    let endpoint =
        "https://www.propertydatamap.com.au/Property/00_PHP_9/Server_Map_SelectSuburb.php";
    let response = client
        .post(endpoint)
        .form(&get_form_data(&suburb_id.to_string()))
        .send()
        .unwrap()
        .json::<RawSelectSuburbResponse>()
        .unwrap();
    if response
        .array_data
        // when there is no voltage lines, the return value becomes an array of some random value
        // instead of an empty object. It's weird but that's what happens.
        .contains("Geometry_Selected_Popup_Info\":[[\"")
    {
        debug!("there is no high voltage power line in {}", suburb_name);
        return SelectSuburbResponse {
            selected_lat_lon: HashMap::new(),
            selected_popup_info: HashMap::new(),
        };
    }
    let array_data = serde_json::from_str::<RawArrayData>(&response.array_data)
        .expect("failed to parse array_data for suburb_id={}");
    let mut selected_lat_lon = HashMap::<String, SelectedLatLon>::new();
    for (k, v) in array_data.geometry_selected_latlon {
        let val = serde_json::from_str::<SelectedLatLon>(&v)
            .expect("failed to parse Geometry_Selected_LatLon");
        selected_lat_lon.insert(k, val);
    }
    SelectSuburbResponse {
        selected_lat_lon,
        selected_popup_info: array_data.geometry_selected_popup_info,
    }
}
