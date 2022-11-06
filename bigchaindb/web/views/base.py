# Copyright © 2020 Interplanetary Database Association e.V.,
# BigchainDB and IPDB software contributors.
# SPDX-License-Identifier: (Apache-2.0 AND CC-BY-4.0)
# Code is Apache-2.0 and docs are CC-BY-4.0

"""Common classes and methods for API handlers
"""
import logging
import time

from flask import jsonify, request

from bigchaindb import config
from bigchaindb.models import Transaction
from bigchaindb.common.exceptions import SchemaValidationError, ValidationError
from bigchaindb.utils import log_metric


logger = logging.getLogger(__name__)


def make_error(status_code, message=None):
    tx = request.get_json(force=True)
    error, tx, tx_obj = validate_schema_definition(tx)
    if status_code == 404 and message is None:
        message = "Not found"

    response_content = {"status": status_code, "message": message}
    request_info = {"method": request.method, "path": request.path}
    request_info.update(response_content)

    logger.error(
        "HTTP API error: %(status)s - %(method)s:%(path)s - %(message)s", request_info
    )
    log_metric(
        "initial_validation_failed",
        tx_obj.metadata["requestCreationTimestamp"],
        tx_obj.operation,
        tx_obj._id,
        None
    )


    response = jsonify(response_content)
    response.status_code = status_code
    return response


def base_ws_uri():
    """Base websocket URL that is advertised to external clients.

    Useful when the websocket URL advertised to the clients needs to be
    customized (typically when running behind NAT, firewall, etc.)
    """

    config_wsserver = config["wsserver"]

    scheme = config_wsserver["advertised_scheme"]
    host = config_wsserver["advertised_host"]
    port = config_wsserver["advertised_port"]

    return "{}://{}:{}".format(scheme, host, port)


def validate_schema_definition(tx):
    # `force` will try to format the body of the POST request even if the
    # `content-type` header is not set to `application/json`
    logger.debug(tx)
    tx_obj, error = None, None
    try:
        tx_obj = Transaction.from_dict(tx)
    except SchemaValidationError as e:
        error = make_error(
            400, message="Invalid transaction schema: {}".format(e.__cause__.message)
        )
    except ValidationError as e:
        error = make_error(
            400, "Invalid transaction ({}): {}".format(type(e).__name__, e)
        )
    return error, tx, tx_obj
