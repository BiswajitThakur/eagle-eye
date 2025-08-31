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
 |               ee-sender                 |
 |    ( running on User 1's device )       |
 +-----------------------------------------+
        |                   ^
        | AES 256           |
        | encrypted         |  AES 256
        | request           |  encrypted
        |                   |  response
        |                   |
        |                   |
        |                   |
        v                   |
 +-----------------------------------------+
 |              ee-receiver                |
 |    ( running on User 2's device )       |
 +-----------------------------------------+
```
