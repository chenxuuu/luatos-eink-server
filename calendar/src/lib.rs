use serde::{Deserialize, Serialize};
/*
"cityid": "101020100",
"city": "上海",
"update_time": "17:34",
"wea": "多云",
"wea_img": "yun",
"tem": "8",
"tem_day": "8",
"tem_night": "4",
"win": "东风",
"win_speed": "1级",
"win_meter": "2km/h",
"air": "29"
格式：https://tianqiapi.com/index/doc?version=day
*/
#[derive(Debug,Serialize, Deserialize)]
struct Weather {
    city: String,
    wea: String,
    wea_img: String,
    tem: String,
    tem_day: String,
    tem_night: String,
    win: String,
    win_speed: String,
    win_meter: String,
    air: String
}
async fn get_weather(location:String, app_id: String, app_secret: String) -> Weather {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().expect("http client build error");
    let resp = client.get(&format!(
        "https://www.yiketianqi.com/free/day?appid={}&appsecret={}&unescape=1&cityid={}",app_id,app_secret,location
    )).send().await.expect("http send error").text().await.expect("http recv error");
    serde_json::from_str(&resp).expect("json decode error")
}


pub async fn get_img_vec(v:u8,location:String, app_id: String, app_secret: String) -> Vec<u8>{
    let weather = get_weather(location,app_id,app_secret).await;
    vec![0x30,0x31,0x32,0x33]
}




// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
