# magic markers

control an led smart bulb using a set of crayola markers! inspired by
[this reel](https://www.instagram.com/reel/DIE2O59Svcz/?igsh=MXNnbmJsZWRmcHhlNA%3D%3D).
the main difference is that this is an entirely self-contained, open source
project that does not require integrating with any smart home systems.

https://github.com/user-attachments/assets/32f9cf9f-bd24-4c65-817b-992e0615c56a

## how it works

each marker has a unique rfid tag, which is read by the nanoc6 rfid reader. the
rfid reader exposes a wifi access point to a lightbulb running tasmota firmware.
color changes post a command directly to the led smart bulb via an http request
to the tasmota command endpoint

## resources

- [tasmota light docs](https://tasmota.github.io/docs/Lights/#3-channels-rgb-lights)
- [tasmota commands](https://tasmota.github.io/docs/Lights/#3-channels-rgb-lights)
- [tasmota firmware](https://github.com/arendst/Tasmota-firmware/tree/firmware/release-firmware/tasmota)
- [nanoc6 examples](https://www.amazon.com/dp/B0B3XQ5Z6F)
- [nanoc6 docs](https://docs.m5stack.com/en/core/M5NanoC6)
- [esp hal wifi embassy access point example](https://github.com/esp-rs/esp-hal/blob/main/examples/src/bin/wifi_embassy_access_point.rs)

## commands

with a fresh tasmota bulb, a template needs to be flashed to it. the below
command configures the bulb and restarts to connect to the esp32 access point.

```bash
backlog template {"NAME":"Kauf Bulb", "GPIO":[0,0,0,0,416,419,0,0,417,420,418,0,0,0], "FLAG":0, "BASE":18, "CMND":"SO105 1|RGBWWTable 204,204,122,153,153"}; module 0; fade 1; devicename magic-markers-bulb; friendlyname1 magic-markers-bulb; ipaddress1 192.168.2.2; ipaddress2 192.168.2.1; ipaddress3 255.255.255.0; ssid1 magic-markers; password1 magic-markers; wificonfig 0
```

## parts

- [ikea fado lamp](https://www.ikea.com/us/en/p/fado-table-lamp-white-70096377/)
- [m5stack nanoc6](https://shop.m5stack.com/products/m5stack-nanoc6-dev-kit?srsltid=AfmBOopeMHd7ymc-D1L89nziaOJ4fDt1PZ01berIM7dOCEl89qVkOAY4)
- [rfid stickers](https://www.amazon.com/Original-Stickers-Rewritable-NFC-Enabled-Smartphones/dp/B0DBQTB6FT/ref=sr_1_6_pp?dib=eyJ2IjoiMSJ9.egpkqpNAVZfPuP6UjMDsn3CU8nARQamCnT0xC5nDJIf5t-uVWoPgwBmoNPKDwBk-PNjjSny202LMEfdwCOKZk3W6Iv6fPljeTY24AGm6G-E6jyqpZe_lnTInHQEeHr6A0njjtCObk__gTJ_l4lzPlSjS-OCnbLXjwuZmz-xA2aUt0lkW5YHW16ou-hvZQ3unhcKs9O9xqlbLgAnMzp2Vnvc9aX7yK6xZXIfJPQiX4497UmzIxfrEmo_Y4jVRxlkN5aplKuS4ImWKjLNZsc0bySEbhd_mO4DY3P0sveg8jcI.YnrOSqJcIs015ueDHAKB04SQx-u8xevfIE7AYaChcfI&dib_tag=se&keywords=rfid+stickers&qid=1757173184&sr=8-6)
- [markers](https://www.amazon.com/dp/B003HGGPLW) -[kauf a21 smart bulb](amazon.com/dp/B09D6HR559?ref_=pe_125775000_1044873430_t_fed_asin_title)
