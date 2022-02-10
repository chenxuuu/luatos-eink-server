use image::{ImageBuffer, GrayImage};
use imageproc::drawing;
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

//屏幕长宽
const WIDTH:u32 = 400;
const HEIGHT:u32 = 300;
//两种颜色
const WHITE:image::Luma<u8> = image::Luma([255]);
const BLACK:image::Luma<u8> = image::Luma([0]);

//生成最终的图片序列
fn generate_eink_bytes(img: &GrayImage)->Vec<u8>{
    let mut r:Vec<u8> = Vec::with_capacity((HEIGHT*WIDTH/8) as usize);//存结果
    for y in 0..HEIGHT {
        for l in 0..WIDTH/8 {
            let mut temp:u8 = 0;
            for i in 0..8 {
                let p:u8 = img.get_pixel(l*8+i,y)[0];
                //匹配像素点颜色
                let t = if p < 127 {0}else{1};
                temp+=t<<(7-i);
            }
            r.push(temp);
        }
    }
    r
}

pub async fn get_img_vec(v:u8,location:String, app_id: String, app_secret: String) -> Vec<u8>{
    //新建个图片当缓冲区
    let mut img: GrayImage = ImageBuffer::new(WIDTH, HEIGHT);
    //刷白
    drawing::draw_filled_rect_mut(&mut img,imageproc::rect::Rect::at(0, 0).of_size(WIDTH, HEIGHT),WHITE);
    //获取天气信息
    let weather = get_weather(location,app_id,app_secret).await;


    generate_eink_bytes(&img)
}




// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }
// }
