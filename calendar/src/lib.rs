use std::io::Read;
use lazy_static::lazy_static;
use image::{ImageBuffer, GrayImage};
use imageproc::drawing;
use rusttype::{Font, Scale};
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
const WIDTH:u32 = 200;
const HEIGHT:u32 = 200;
//两种颜色
const WHITE:image::Luma<u8> = image::Luma([255]);
const BLACK:image::Luma<u8> = image::Luma([0]);

//获取资源文件路径
pub fn get_path() -> String{
    let path = std::env::args().nth(2).unwrap_or(String::from("static/"));
    if path.ends_with('/') || path.ends_with('\\') {
        path
    } else {
        path + "/"
    }
}
//加载字体文件
fn load_font(path: String) -> Font<'static>{
    let mut file = std::fs::File::open(&path).expect(&path);
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    Font::try_from_vec(data).unwrap()
}
//静态加载字体
lazy_static! {
    static ref FONT_MI: Font<'static> = load_font(get_path() + "MiSans-Regular.ttf");
}

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

    drawing::draw_text_mut(&mut img, BLACK, 0,0, Scale {x: 22.0,y: 22.0 }, &FONT_MI,"测试");

    //返回图片数据
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
