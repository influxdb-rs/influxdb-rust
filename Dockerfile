FROM xd009642/tarpaulin

RUN wget https://dl.influxdata.com/influxdb/releases/influxdb_1.7.6_amd64.deb
RUN dpkg -i influxdb_1.7.6_amd64.deb
RUN INFLUXDB_HTTP_BIND_ADDRESS=9999 influxd > $HOME/influx.log 2>&1 &

WORKDIR /volume

CMD cargo build && cargo tarpaulin