FROM bitcoin/bitcoin:28.1-alpine
COPY bitcoin.conf /root/.bitcoin/bitcoin.conf
COPY entry_point.sh /entry_point.sh
RUN chmod +x /entry_point.sh
ENTRYPOINT /entry_point.sh

