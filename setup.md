# Setup

Clone the repo:
`
cd /home
sudo apt-get install git-lfs
git lfs install
git clone git@github.com:BigBadE/BeatmapBrowser.git
`

Install Rust: 
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Install C++:
`
sudo apt-get install clang
sudo apt-get install g++
sudo apt-get install build-essential
sudo apt-get install gcc
`

Install surrealdb:
`
sudo apt-get install surrealdb
`

Setup the database service:
`
sudo adduser --system --no-create-home --group surrealdb
sudo mkdir /etc/surrealdb
sudo chown -R surrealdb:surrealdb /etc/surrealdb
sudo nano /etc/systemd/system/surrealdb.service
`

Surreal service:
`
[Unit]
Description=Backend Database Service
After=network.target

[Service]
Type=simple
User=surrealdb
Group=surrealdb
WorkingDirectory=/etc/surrealdb
ExecStart=surreal start --user root --pass root rocksdb:/etc/surrealdb/db
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
`

Run it:
`
sudo systemctl daemon-reload
sudo systemctl enable surrealdb.service
`

Setup user and group:
`
sudo adduser --system --no-create-home --group rustapp
sudo nano /etc/systemd/system/rustapp.service
`

Paste in the service:
`
[Unit]
Description=Rust Backend Service
After=network.target

[Service]
User=rustapp
Group=rustapp
WorkingDirectory=/home/BeatmapBrowser
ExecStart=/home/BeatmapBrowser/target/release/backend 127.0.0.1:3000
Restart=on-failure

[Install]
WantedBy=multi-user.target
`

Run it:
`
sudo systemctl daemon-reload
sudo systemctl enable rustapp.service
`

View logs:
`
sudo journalctl -u rustapp.service -f
`

Install nginx:
`
sudo apt-get install nginx
`

Put cert and private key in:
`
sudo nano /etc/ssl/certs/www.beatblockbrowser.crt
sudo nano /etc/ssl/private/www.beatblockbrowser.key
`

Set perms:
`
sudo chown root:root /etc/ssl/certs/www.beatblockbrowser.crt
sudo chown root:root /etc/ssl/private/www.beatblockbrowser.key
sudo chmod 600 /etc/ssl/private/www.beatblockbrowser.key
`

Configure nginx:
`
sudo nano /etc/nginx/sites-available/www.beatblockbrowser.conf
`

`
server {
    listen 80;
    server_name beatblockbrowser.me www.beatblockbrowser.me;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name beatblockbrowser.me www.beatblockbrowser.me;

    ssl_certificate     /etc/ssl/certs/www.beatblockbrowser.crt;
    ssl_certificate_key /etc/ssl/private/www.beatblockbrowser.key;

    # SSL Settings
    ssl_session_cache   shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_prefer_server_ciphers on;
    keepalive_timeout   70;

    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    resolver 8.8.8.8 8.8.4.4 valid=300s;
    resolver_timeout 5s;

    # HSTS (Optional but recommended)
    add_header Strict-Transport-Security "max-age=63072000; includeSubdomains; preload" always;

    # Additional Security Headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Referrer-Policy "no-referrer-when-downgrade";
    add_header Permissions-Policy "geolocation=()";
    add_header Content-Security-Policy "
        default-src 'self';
        script-src 'self' https://code.jquery.com;
        style-src 'self' https://cdn.jsdelivr.net https://fonts.googleapis.com;
        font-src 'self' https://fonts.gstatic.com;
        img-src 'self' https://images.example.com data:;
        connect-src 'self' https://api.example.com;
        frame-src 'none';
        object-src 'none';
        base-uri 'self';
        form-action 'self';
        upgrade-insecure-requests;
    ";

    # Proxy Settings
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
`

Link the site, run it, and allow firewall access:
`
sudo rm /etc/nginx/sites-enabled/default
sudo ln -s /etc/nginx/sites-available/www.beatblockbrowser.conf /etc/nginx/sites-enabled/
sudo systemctl reload nginx
sudo ufw allow 'Nginx Full'
`

Setup the webhooks:
`
sudo apt install webhook
sudo mkdir /etc/webhook
sudo nano /etc/webhook/hooks.json
`

Add the hook:
`
[
  {
    "id": "deploy-webhook",
    "execute-command": "/home/BeatmapBrowser/deploy.sh",
    "command-working-directory": "/home/BeatmapBrowser",
    "response-message": "Deploying...",
    "trigger-rule": {
      "and": [
      {
        "match": {
          "type": "payload-hmac-sha256",
          "secret": "124305912678e7d834448a3c461366eb",
          "parameter": {
            "source": "header",
            "name": "X-Hub-Signature-256"
          }
        }
      },
      {
        "match": {
          "type": "value",
          "value": "refs/heads/master",
          "parameter": {
            "source": "payload",
            "name": "ref"
          }
        }
      }
      ]
    }
  }
]
`

Configure webhooks:
`
sudo nano /etc/webhook.conf
`

Setup the nginx for the webhook server:
`
sudo nano /etc/nginx/sites-available/webhook.beatblockbrowser.conf
`

Configuration:
`
server {
    listen 80;
    server_name webhook.beatblockbrowser.me www.webhook.beatblockbrowser.me;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    server_name webhook.beatblockbrowser.me www.webhook.beatblockbrowser.me;

    ssl_certificate     /etc/ssl/certs/www.beatblockbrowser.crt;
    ssl_certificate_key /etc/ssl/private/www.beatblockbrowser.key;

    # SSL Settings
    ssl_session_cache   shared:SSL:10m;
    ssl_session_timeout 10m;
    ssl_prefer_server_ciphers on;
    keepalive_timeout   70;

    # OCSP Stapling
    ssl_stapling on;
    ssl_stapling_verify on;
    resolver 8.8.8.8 8.8.4.4 valid=300s;
    resolver_timeout 5s;

    # HSTS (Optional but recommended)
    add_header Strict-Transport-Security "max-age=63072000; includeSubdomains; preload" always;

    # Additional Security Headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Referrer-Policy "no-referrer-when-downgrade";
    add_header Permissions-Policy "geolocation=()";

    # Proxy Settings
    location / {
        proxy_pass http://127.0.0.1:9000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
`

Now enable it and run the webhook:
`
sudo ln -s /etc/nginx/sites-available/webhook.beatblockbrowser.conf /etc/nginx/sites-enabled/
sudo adduser --system --no-create-home --group webhook
sudo mkdir -p /var/lib/webhook
sudo chown webhook:webhook /var/lib/webhook
sudo chmod 750 /var/lib/webhook
sudo mkdir -p /var/lib/webhook/.ssh
sudo chown webhook:webhook /var/lib/webhook/.ssh
sudo chmod 700 /var/lib/webhook/.ssh
sudo -u webhook ssh-keygen -t rsa -b 4096 -C "webhook@beatblockbrowser.me" -f /var/lib/webhook/.ssh/id_rsa -N ""
sudo chmod 600 /var/lib/webhook/.ssh/id_rsa
sudo chmod 644 /var/lib/webhook/.ssh/id_rsa.pub
sudo chown -R webhook:webhook /etc/webhook
sudo chown -R webhook:webhook /home/BeatmapBrowser

Get your SSH key and add them to the github:
`
sudo cat /var/lib/webhook/.ssh/id_rsa.pub
`

Setup the webhook service:
`
sudo nano /etc/systemd/system/webhook.service
`


Add the webhook service:
`
[Unit]
Description=Webhook Listener Service
After=network.target

[Service]
Type=simple
User=webhook
Group=webhook
WorkingDirectory=/etc/webhook
ExecStart=webhook -verbose -hooks /etc/webhook/hooks.json -port 9000
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
`

And run it:
`
sudo systemctl daemon-reload
sudo systemctl enable webhook.service
`