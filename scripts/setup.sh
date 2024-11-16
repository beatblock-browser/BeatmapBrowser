. config/config.config

# Secure certs
echo $SITE_CERT >> /etc/ssl/certs/site.crt
chown root:root /etc/ssl/certs/site.crt
echo $SITE_KEY >> /etc/ssl/private/site.key
chown root:root /etc/ssl/private/site.key
chmod 600 /etc/ssl/private/site.key

# Setup site
cp config/sites-available/*.conf /etc/nginx/sites-available/
sed -i -e "s/{DOMAIN}/$DOMAIN/g" /etc/nginx/sites-available/*.conf

# Setup nginx
rm /etc/nginx/sites-enabled/default
ln -s /etc/nginx/sites-available/www.site /etc/nginx/sites-enabled/

surrealdb start --bin 127.0.0.1:8000 &
nginx &

exec /usr/local/bin/backend