using RestSharp;
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Data;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace picDataConvert
{
    public partial class Form1 : Form
    {
        public Form1()
        {
            InitializeComponent();
        }


        private static Bitmap outBmp = new Bitmap(1000, 1000);
        private static Graphics g = Graphics.FromImage(outBmp);
        private void button2_Click(object sender, EventArgs e)
        {
            var client = new RestClient();
            if(checkBox1.Checked)
                client.BaseUrl = new Uri("http://127.0.0.1:23366/luatos-calendar/v1");
            else
                client.BaseUrl = new Uri("https://qq.papapoi.com/luatos-calendar/v1");
            var request = new RestRequest(RestSharp.Method.GET);
            request.AddParameter("mac", "test");
            request.AddParameter("battery", "50");
            request.AddParameter("location", "101020100");
            request.AddParameter("appid", "27548549");
            request.AddParameter("appsecret", "rEi9nRak");
            var response = client.Execute(request);

            if(response.RawBytes == null)
            {
                MessageBox.Show($"获取失败:{response.ErrorMessage}");
                return;
            }

            for (int i = 0; i < response.RawBytes.Length; i++)
            {
                int now1 = response.RawBytes[i];
                int y = i / 25;
                int x = i % 25 * 8;
                for (int b = 0; b < 8; b++)
                {
                    int bit = (now1 >> (7 - b)) & 1;
                    switch (bit)
                    {
                        case 0:
                            g.FillRectangle(Brushes.Black, (x + b) * 5, y * 5, 5, 5);
                            break;
                        case 1:
                            g.FillRectangle(Brushes.White, (x + b) * 5, y * 5, 5, 5);
                            break;
                    }
                }
            }
            pictureBox2.Image = outBmp;
        }

    }
}
