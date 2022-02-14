use std::{io::Read, vec};
use chrono::{DateTime, Datelike, NaiveDate};
use lazy_static::lazy_static;
use image::{ImageBuffer, GrayImage};
use imageproc::drawing;
use log::info;
use rusttype::{Font, Scale};
use serde::{Deserialize, Serialize};
use lunardate::LunarDate;

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
//获取天气图片
fn get_weather_img(name: &str, size: u32) -> ImageBuffer<image::Luma<u8>, Vec<u8>> {
    let size = size.to_string();
    let file = match image::open(get_path()+"icons/"+&size+"/"+&name.to_string()+".png") {
        Ok(file) => file,
        Err(_) => {
            println!("not found!{}",name);
            image::open(get_path()+"icons/"+&size+"/404.png").unwrap()
        },
    };
    file.into_luma8()
}
//静态加载字体
lazy_static! {
    static ref FONT_SARASA: Font<'static> = load_font(get_path() + "sarasa-mono-sc-nerd-regular.ttf");
    static ref FONT_PIXEL: Font<'static> = load_font(get_path() + "LanaPixel.ttf");
    static ref FONT_SOURCE: Font<'static> = load_font(get_path() + "SourceHanSansCN-Regular.ttf");
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
    //画上星期
    drawing::draw_filled_rect_mut(img, imageproc::rect::Rect::at(x as i32, y as i32).of_size(63, 9), BLACK);
    put_calender_font(img,16,x+1,y+1);//周日
    for i in 10..16 {//周一到周六
        put_calender_font(img,i,1+9+x+((i-10)*9) as u32,y+1);
    }
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

const LUNAR_MONTH : [&str; 12] = ["正","二","三","四","五","六","七","八","九","十","冬","腊"];
const LUNAR_DAY : [&str; 30] = ["初一","初二","初三","初四","初五","初六","初七","初八","初九","初十","十一","十二","十三","十四","十五",
"十六","十七","十八","十九","二十","廿一","廿二","廿三","廿四","廿五","廿六","廿七","廿八","廿九","三十"];
const MONTH_NAME : [&str; 12] = ["一月","二月","三月","四月","五月","六月","七月","八月","九月","十月","十一月","十二月"];
const WEEK_NAME : [&str; 7] = ["一","二","三","四","五","六","日"];
//获取当天的农历日期
fn get_lunar(time: &LunarDate) -> String {
    let mut r = String::from("农历");
    r.push_str(LUNAR_MONTH[time.month() as usize - 1]);
    r.push_str("月");
    r.push_str(LUNAR_DAY[time.day() as usize - 1]);
    r
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
async fn get_weather_day(location: &str, app_id: &str, app_secret: &str) -> Weather {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().expect("http client build error");
    let resp = client.get(&format!(
        "https://www.yiketianqi.com/free/day?appid={}&appsecret={}&unescape=1&cityid={}",app_id,app_secret,location
    )).send().await.expect("http send error").text().await.expect("http recv error");
    serde_json::from_str(&resp).expect("json decode error")
}

//画当前天气
fn put_weather_day(img: &mut ImageBuffer<image::Luma<u8>, Vec<u8>>, w: &Weather) {
    //图标
    let icon = get_weather_img(&w.wea_img,80);
    image::imageops::overlay(img, &icon, 6, 3);
    //天气文字
    drawing::draw_text_mut(img, BLACK, 60,75, Scale {x: 30.0,y: 30.0 }, &FONT_SARASA,&w.wea);
    //温度
    let offset = match w.tem.len() {
        1 => 20,
        2 => 10,
        _ => 0
    };
    let offset2 = match w.tem.len() {
        1 => 40,
        2 => 50,
        _ => 60
    };
    drawing::draw_text_mut(img, BLACK, offset,80, Scale {x: 50.0,y: 50.0 }, &FONT_SARASA,&w.tem);
    drawing::draw_text_mut(img, BLACK, offset2,98, Scale {x: 25.0,y: 25.0 }, &FONT_SOURCE,"℃");
    //空气质量
    let mut color = BLACK;
    if w.air.parse::<u16>().unwrap() > 100 {
        color = WHITE;
        drawing::draw_filled_rect_mut(img, imageproc::rect::Rect::at(79, 103).of_size(41, 21), BLACK);
    }
    drawing::draw_text_mut(img, color, 80,104, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,"空气质量");
    drawing::draw_text_mut(img, color, 80,114, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,"指数:");
    drawing::draw_text_mut(img, color, 103,114, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,&w.air);
}

/*
"cityid":"101120101",
"city":"济南",
"update_time":"2020-04-21 17:24:11",
"data":[
    {
        "date":"2020-04-21",
        "wea":"晴",
        "wea_img":"qing",
        "tem_day":"17",
        "tem_night":"4",
        "win":"北风",
        "win_speed":"3-4级"
    },
    ...
格式：https://tianqiapi.com/index/doc?version=week
*/
#[derive(Debug,Serialize, Deserialize)]
struct WeatherWeek {
    city: String,
    data: Vec<WeatherWeekDay>
}
#[derive(Debug,Serialize, Deserialize)]
struct WeatherWeekDay {
    date: String,    
    wea: String,
    wea_img: String,
    tem_day: String,
    tem_night: String,
    win: String,
    win_speed: String,
}
async fn get_weather_week(location: &str, app_id: &str, app_secret: &str) -> WeatherWeek {
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(5)).build().expect("http client build error");
    let resp = client.get(&format!(
        "https://www.yiketianqi.com/free/week?appid={}&appsecret={}&unescape=1&cityid={}",app_id,app_secret,location
    )).send().await.expect("http send error").text().await.expect("http recv error");
    serde_json::from_str(&resp).expect("json decode error")
}
//画当前天气
fn put_weather_week(img: &mut ImageBuffer<image::Luma<u8>, Vec<u8>>, w: &WeatherWeek) {
    //图标
    image::imageops::overlay(img, &get_weather_img(&w.data[0].wea_img,40), 4, 136);
    image::imageops::overlay(img, &get_weather_img(&w.data[1].wea_img,40), 4 + 40 + 4, 136);
    image::imageops::overlay(img, &get_weather_img(&w.data[2].wea_img,40), 4 + 40 + 4 + 40 + 4, 136);
    //三天
    drawing::draw_filled_rect_mut(img, imageproc::rect::Rect::at(0, 126).of_size(134, 11), BLACK);
    drawing::draw_text_mut(img, WHITE, 4 + 10,127, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,"今天");
    drawing::draw_text_mut(img, WHITE, 4 + 40 + 4 + 10,127, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,"明天");
    drawing::draw_text_mut(img, WHITE, 4 + 40 + 4 + 40 + 4 + 10,127, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,"后天");
    //温度
    drawing::draw_text_mut(img, BLACK, 2,176, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,"日");
    drawing::draw_text_mut(img, BLACK, 2,188, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,"夜");
    let tem_img = image::open(get_path()+"tem.png").unwrap().to_luma8();
    image::imageops::overlay(img, &tem_img, 4 + 28, 177);
    image::imageops::overlay(img, &tem_img, 4 + 28, 189);
    image::imageops::overlay(img, &tem_img, 4 + 40 + 4 + 28, 177);
    image::imageops::overlay(img, &tem_img, 4 + 40 + 4 + 28, 189);
    image::imageops::overlay(img, &tem_img, 4 + 40 + 4 + 40 + 4 + 28, 177);
    image::imageops::overlay(img, &tem_img, 4 + 40 + 4 + 40 + 4 + 28, 189);
    drawing::draw_text_mut(img, BLACK, 4 + 12,176, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,&w.data[0].tem_day);
    drawing::draw_text_mut(img, BLACK, 4 + 40 + 4 + 12,176, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,&w.data[1].tem_day);
    drawing::draw_text_mut(img, BLACK, 4 + 40 + 4 + 40 + 4 + 12,176, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,&w.data[2].tem_day);
    drawing::draw_text_mut(img, BLACK, 4 + 12,188, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,&w.data[0].tem_night);
    drawing::draw_text_mut(img, BLACK, 4 + 40 + 4 + 12,188, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,&w.data[1].tem_night);
    drawing::draw_text_mut(img, BLACK, 4 + 40 + 4 + 40 + 4 + 12,188, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,&w.data[2].tem_night);
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
                let t = if p > 127 {0}else{1};
                temp+=t<<i;
            }
            r.push(temp);
        }
    }
    r
}

pub async fn get_img_vec(v:u8,location:String, app_id: String, app_secret: String) -> Vec<u8>{
    //获取当前时间
    let time_now = chrono::Local::now();
    let lunar_now = LunarDate::from_solar_date(time_now.year(),time_now.month(),time_now.day()).unwrap();
    //新建个图片当缓冲区
    let mut img: GrayImage = ImageBuffer::new(WIDTH, HEIGHT);
    //刷白
    drawing::draw_filled_rect_mut(&mut img,imageproc::rect::Rect::at(0, 0).of_size(WIDTH, HEIGHT),WHITE);

    ///////////////// 日历部分 ///////////////////
    //日期
    drawing::draw_text_mut(&mut img, BLACK, 100,0, Scale {x: 121.0,y: 121.0 }, &FONT_SARASA,
        &time_now.day().to_string());
    //年月
    drawing::draw_text_mut(&mut img, BLACK, 107,1, Scale {x: 21.0,y: 21.0 }, &FONT_SARASA,
        &(time_now.year().to_string()+"年 "+&time_now.month().to_string()+"月")
    );
    //星期
    drawing::draw_text_mut(&mut img, BLACK, 120,98, Scale {x: 30.0,y: 30.0 }, &FONT_SARASA,
        &("星期".to_owned()+WEEK_NAME[time_now.weekday().num_days_from_monday() as usize]));
    //农历时间
    drawing::draw_text_mut(&mut img, BLACK, 2 + 135,128, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,&get_lunar(&lunar_now));
    //日历的月份
    let offset = if time_now.month() > 10 {19 + 135} else {22 + 135};
    drawing::draw_text_mut(&mut img, BLACK, offset,140, Scale {x: 11.0,y: 11.0 }, &FONT_PIXEL,
        MONTH_NAME[time_now.month0() as usize]);
    //日历
    put_calender(&mut img,&time_now,135,150);

    ///////////////// 天气部分 ///////////////////////
    //今日天气信息
    put_weather_day(&mut img,&get_weather_day(&location,&app_id,&app_secret).await);
    //三天天气信息
    put_weather_week(&mut img,&get_weather_week(&location,&app_id,&app_secret).await);

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
