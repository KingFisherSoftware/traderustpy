# RUst - MAturin - pyO3  -- github.com/kfsone/rumao3 / hub.docker.com/kfsone/rumao3
#
# See README.md for the purpose the project.

# Start from a simple Rust image, skip fancy trimmings.
FROM rust:slim AS baseline

# It's expected you'll mount the parent-folder of your project (typically this directory)
# as /src. e.g. docker build --tag kfsone/rumao3 -v ${PWD}:/src
VOLUME  /src
WORKDIR /src

# If you have your own apt-cacher or proxy, specify it here or on the command line,
# e.g. I use: docker build --tag kfsone/rumao3 --build-arg APT_PROXY=http://wafer.lan:3142/
ARG APT_PROXY=""
ENV APT_PROXY="${APT_PROXY}"

# Enable the inclusion of editors inside the container. Set to 0 to disable
ARG WITH_EDITORS="0"
ENV WITH_EDITORS="${WITH_EDITORS}"

# Specify the python we're using
ARG TARGET_PY="python3"
ENV TARGET_PY="${TARGET_PY}"

# In a single layer, enable the Apt proxy to increase build-speed if one is available,
# get apt into a good state, and then install a few baseline packages to make it possible
# to work inside the image.
#
# Then purge the cache and clean up the apt lists etc to minimize layer/image size.
RUN if [ -n "${APT_PROXY}" ]; then \
		echo "Acquire::Http::Proxy { \"${APT_PROXY}\"; }" >/etc/apt/apt.conf.d/02proxy; \
	fi ; \
	apt update && \
	apt install -qy "${TARGET_PY}" python-is-python3 python3-pip python3-virtualenv && \
	if [ "${WITH_EDITORS:-1}" != "0" ]; then apt install -qy nano vim neovim; else true; fi && \
	apt clean cache -y && apt autoclean -qy && \
	rm -rf /var/lib/apt/lists/* /etc/apt/apt.conf.d/02proxy


# Create a python virtualenv in a discrete, discardable layer
FROM baseline as python
RUN	\
	"${TARGET_PY}" -m virtualenv /var/venv && \
	. /var/venv/bin/activate && \
	python -m pip install --no-cache --upgrade pip wheel setuptools


# Final layer: install maturin and lets give it its final name.
FROM baseline AS rumao3

# Make the virtualenv path part of the path
ENV PATH="/root/.venv/bin:${PATH}"

# Copy the virtual env over without baggage
COPY --from=python /var/venv /var/venv
RUN . /var/venv/bin/activate && cargo install --no-default-features --features full maturin

# Install a copy of the sample into /opt/sample
COPY sample /opt/sample

CMD [ "/bin/sh", "-c", "bash --init-file /var/venv/bin/activate" ]
