use std::{io::Read, vec};
use chrono::{DateTime, Datelike, NaiveDate};
use lazy_static::lazy_static;
use image::{ImageBuffer, GrayImage};
use imageproc::drawing;
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};


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
//加载显示日历用的雪碧图
fn load_sprit() -> Vec<ImageBuffer<image::Luma<u8>, Vec<u8>>> {
    let mut r = Vec::with_capacity(10+7);
    let file = image::open(get_path()+"pixelNumbers.png").unwrap();
    for i in 0..10 {
        r.push(file.clone().crop(i*3, 0, 3, 5).into_luma8())
    }
    for i in 0..7 {
        r.push(file.clone().crop(30+i*7, 0, 7, 7).into_luma8())
    }
    //白色数字（反色）
    let file = image::open(get_path()+"pixelNumbersB.png").unwrap();
    for i in 0..10 {
        r.push(file.clone().crop(i*3, 0, 3, 5).into_luma8())
    }
    r
}
//静态加载字体
lazy_static! {
    static ref FONT_MI: Font<'static> = load_font(get_path() + "MiSans-Regular.ttf");
    static ref FONT_CALE: Vec<ImageBuffer<image::Luma<u8>, Vec<u8>>> = load_sprit();
}


//放置日历字体
fn put_calender_font(img: &mut ImageBuffer<image::Luma<u8>, Vec<u8>>,index: usize, x: u32, y: u32) {
    image::imageops::overlay(img, &FONT_CALE[index], x, y);
}
//放置日历数字 宽度3，每个数字间隔1，总宽7
fn put_calender_num(img: &mut ImageBuffer<image::Luma<u8>, Vec<u8>>,num: u32, x: u32, y: u32, color: image::Luma<u8>) {
    let n = if color == WHITE {17} else {0};
    image::imageops::overlay(img, &FONT_CALE[(num / 10 + n) as usize], x, y);
    image::imageops::overlay(img, &FONT_CALE[(num % 10 + n) as usize], x+4, y);
}
//放置日历，显示传入日期
fn put_calender(img: &mut ImageBuffer<image::Luma<u8>, Vec<u8>>,time: &DateTime<chrono::Local>, x: u32, y: u32) {
    let now = time.day();//当前日期
    let start = time.with_day(1).unwrap().weekday().num_days_from_sunday();//本月第一天的日期
    //这个月有多少天 https://stackoverflow.com/questions/53687045
    let max = if time.month() == 12 {
            NaiveDate::from_ymd(time.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd(time.year(), time.month() + 1, 1)
        }.signed_duration_since(NaiveDate::from_ymd(time.year(), time.month(), 1))
        .num_days() as u32;
    put_calender_font(img,16,x+1,y);//周日
    for i in 10..16 {//周一到周六
        put_calender_font(img,i,1+9+x+((i-10)*9) as u32,y);
    }
    //画一根线
    drawing::draw_line_segment_mut(img, (x as f32,(y+8) as f32), ((x+7*9-1) as f32,(y+8) as f32), BLACK);
    //每个日期都画上去
    for i in 0..max {
        let place = i + start;
        let nx = place % 7;
        let ny = place / 7;
        if now-1 == i {//日期是当天就突出显示
            drawing::draw_filled_rect_mut(img,
                imageproc::rect::Rect::at((x+nx*9).try_into().unwrap(), (y+9+ny*7).try_into().unwrap()).of_size(9, 7),
                BLACK);
            put_calender_num(img,i+1,x+1+nx*9,y+10+ny*7, WHITE);
        } else {
            put_calender_num(img,i+1,x+1+nx*9,y+10+ny*7,BLACK);
        }
    }
}

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
    //获取当前时间
    let time_now = chrono::Local::now();
    //新建个图片当缓冲区
    let mut img: GrayImage = ImageBuffer::new(WIDTH, HEIGHT);
    //刷白
    drawing::draw_filled_rect_mut(&mut img,imageproc::rect::Rect::at(0, 0).of_size(WIDTH, HEIGHT),WHITE);
    //获取天气信息
    let weather = get_weather(location,app_id,app_secret).await;

    //drawing::draw_text_mut(&mut img, BLACK, 0,0, Scale {x: 22.0,y: 22.0 }, &FONT_MI,"测试");

    put_calender(&mut img,&time_now,10,10);

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
