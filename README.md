# Eagle EYE

## UNDER DEVELOPMENT

Access storage, camera, execute cmd, install application on your other devices connected in same network.

```
 +-----------------------------------------+
 |             User 1                      |
 | (eagle-eye-cli | browser | curl | ... ) |
 +-----------------------------------------+
        |                  ^
        |                  |
   http | request     http | response
        |                  |
        v                  |
 +-----------------------------------------+
 |           eagle-eye-daemon              |
 |    ( running on User 1's device )       |
 +-----------------------------------------+
        |                   ^
        | AES 256           |
        | encrypted         |
        | eagle-eye         |  AES 256
        | request           |  encrypted
        |     +             |  eagle-eye
        | ( access key      |  response
        | of User 2's       |
        |   device )        |
        |                   |
        v                   |
 +-----------------------------------------+
 |           eagle-eye-daemon              |
 |    ( running on User 2's device )       |
 +-----------------------------------------+
```
