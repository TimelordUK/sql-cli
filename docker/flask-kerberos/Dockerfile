FROM python:3.11-slim

# Install Kerberos client libraries
RUN apt-get update && apt-get install -y \
    krb5-user \
    libkrb5-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Python requirements and install
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy Kerberos renewal script
COPY kerberos_renew.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/kerberos_renew.sh

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Copy Flask application
COPY app.py .

# Create log directory
RUN mkdir -p /var/log && chmod 755 /var/log

# Expose Flask port
EXPOSE 5000

# Set entrypoint
ENTRYPOINT ["/entrypoint.sh"]
