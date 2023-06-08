# docker run -v /var/lib/nanocl/store/certs:/certs -v /var/lib/nanocl/store/ca:/ca -it --rm cockroachdb/cockroach:v22.2.7 cert create-ca --certs-dir=/certs --ca-key=/ca/ca.key
# docker run -v /var/lib/nanocl/store/certs:/certs -v /var/lib/nanocl/store/ca:/ca -it --rm cockroachdb/cockroach:v22.2.7 cert create-node nstore.nanocl.internal --certs-dir=/certs --ca-key=/ca/ca.key
# docker run -v /var/lib/nanocl/store/certs:/certs -v /var/lib/nanocl/store/ca:/ca -it --rm cockroachdb/cockroach:v22.2.7 cert create-client root --certs-dir=/certs --ca-key=/ca/ca.key
# docker run --network system -v /var/lib/nanocl/store/certs:/certs -v /var/lib/nanocl/store/ca:/ca -it --rm cockroachdb/cockroach:v22.2.7 sql --certs-dir=/certs --host=nstore.nanocl.internal:26257
# docker run --network system -v /var/lib/nanocl:/var/lib/nanocl -it --rm jbergknoff/postgresql-client "postgresql://root@nstore.nanocl.internal:26257/defaultdb?sslcert=/var/lib/nanocl/store/certs/client.root.crt&sslkey=/var/lib/nanocl/store/certs/client.root.key&sslmode=require&sslrootcert=/var/lib/nanocl/store/certs/ca.crt"
