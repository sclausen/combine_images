curl -X POST http://127.0.0.1:8000/process \
-H "Content-Type: application/json" \
-d '{"background": "http://127.0.0.1:8000/images/background.png", "overlay": "http://127.0.0.1:8000/images/overlay.png"}'