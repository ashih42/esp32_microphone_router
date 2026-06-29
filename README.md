# ESP32 Microphone System (Using ESP IDF)

Description goes here.

[My beautiful diagram](https://drive.google.com/drive/u/0/folders/1rg2EKlrOoOMn79tE5XOSEtZ22IKvavUq)

## Developer Notes

### .env

You must include these in your `.env` file:

```
# These must be exactly 16 chars.
ESP_NOW_PMK=****************
ESP_NOW_LMK=****************
```

These 2 passwords are used to encrypt messages sent over ESP-NOW.

### To monitor the output

#### (1) Use `espflash monitor`

```
espflash monitor --baud 115200 --chip esp32
```

This is slow to connect, and often it just fails to connect.

#### (2) Use `screen`

```
TERM=xterm screen /dev/cu.usbserial-0001 115200
```

This always works immediately.

Note: Setting `$TERM` is necessary on my machine because the default name (`xterm-256color`) is too long for `screen` to work with.

To exit, press `Ctrl`+`A`, then `K`, and then `Y`.

## References

- [ESP-NOW Protocol](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/network/esp_now.html)

- [ESP32 Pinout Diagram](https://www.teachmemicro.com/esp32-pinout-diagram-wroom-32/)

- [The exact ESP32 devboards I'm using](https://www.amazon.com/dp/B0CNYK7WT2)
  - Why you put pin labels on the bottom (╯°□°)╯︵ ┻━┻
