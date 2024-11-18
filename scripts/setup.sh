#!/usr/bin/env bash
if [ -f /usr/local/config/config.config ]; then
  . /usr/local/config/config.config
else
  . /usr/local/config/example.config

# Secure certs
echo $SITE_CERT >> /etc/ssl/certs/site.crt
chown root:root /etc/ssl/certs/site.crt
echo $SITE_KEY >> /etc/ssl/private/site.key
chown root:root /etc/ssl/private/site.key
chmod 600 /etc/ssl/private/site.key

# Setup site
cp /usr/local/config/sites-available/*.conf /etc/nginx/sites-available/
sed -i -e "s/{DOMAIN}/$DOMAIN/g" /etc/nginx/sites-available/*.conf

# Setup nginx
rm /etc/nginx/sites-enabled/default
ln -s /etc/nginx/sites-available/www.site.conf /etc/nginx/sites-enabled/www.site.conf

# Setup surrealdb
curl -sSf https://install.surrealdb.com | sh

echo "Running surrealdb"
surreal start --user root --pass root "rocksdb:/usr/local/db" &
sleep 5

nginx &

exec /usr/local/bin/backend 127.0.0.1:3000