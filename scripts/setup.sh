. config/config.config

echo "Writing certs"
echo $SITE_CERT >> /etc/ssl/certs/site.crt
sudo chown root:root /etc/ssl/certs/site.crt
echo $SITE_KEY >> /etc/ssl/private/site.key
sudo chown root:root /etc/ssl/private/site.key
chmod 600 /etc/ssl/private/site.key

echo "Setting up nginx site"
cp config/sites-available/*.conf /etc/nginx/sites-available/
sed -i -e "s/{DOMAIN}/$DOMAIN/g" /etc/nginx/sites-available/*.conf

echo "Enabling site"
rm /etc/nginx/sites-enabled/default
ln -s /etc/nginx/sites-available/www.site /etc/nginx/sites-enabled/

echo "Starting nginx"
sudo systemctl reload nginx
sudo ufw allow 'Nginx Full'

echo "Starting surrealdb"
surrealdb start --bin 127.0.0.1:8000 &

echo "Starting the website"
exec /usr/local/bin/backend