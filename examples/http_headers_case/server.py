from flask import Flask, request

app = Flask(__name__)

@app.route('/headers', methods=['GET', 'POST'])
def headers():
    headers = request.headers
    headers_dict = {key: value for key, value in headers.items()}
    print(headers_dict)
    return headers_dict, 200

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)