# Copyright © 2020 Interplanetary Database Association e.V.,
# BigchainDB and IPDB software contributors.
# SPDX-License-Identifier: (Apache-2.0 AND CC-BY-4.0)
# Code is Apache-2.0 and docs are CC-BY-4.0

"""This module provides the blueprint for some basic API endpoints.

For more information please refer to the documentation: http://bigchaindb.com/http-api
"""
import logging

from flask import current_app, request, jsonify
from flask_restful import Resource, reqparse

from bigchaindb.common.transaction_mode_types import (
    BROADCAST_TX_ASYNC,
    BROADCAST_TX_SYNC,
    BROADCAST_TX_COMMIT,
)
from bigchaindb.common.exceptions import SchemaValidationError, ValidationError
from bigchaindb.web.views.base import make_error, validate_schema_definition
from bigchaindb.web.views import parameters
from bigchaindb.models import Transaction
from bigchaindb.utils import log_metric


logger = logging.getLogger(__name__)
recovery_logger = logging.getLogger("recovery")


class TransactionApi(Resource):
    def get(self, tx_id):
        """API endpoint to get details about a transaction.

        Args:
            tx_id (str): the id of the transaction.

        Return:
            A JSON string containing the data about the transaction.
        """
        pool = current_app.config["bigchain_pool"]

        with pool() as bigchain:
            tx = bigchain.get_transaction(tx_id)

        if not tx:
            return make_error(404)

        return tx.to_dict()


class TransactionValidateApi(Resource):
    def get(self):
        return dict()

    def post(self):
        """API endpoint to validate transaction.

        Return:
            A ``dict`` containing the data about the transaction.
        """
        pool = current_app.config["bigchain_pool"]

        error, tx, tx_obj = validate_schema_definition(request)

        if error is not None:
            return error

        with pool() as bigchain:
            try:
                bigchain.validate_transaction(tx_obj)
            except ValidationError as e:
                return make_error(
                    400, "Invalid transaction ({}): {}".format(type(e).__name__, e)
                )

        response = jsonify(tx)
        response.status_code = 202
        return response


class TransactionListApi(Resource):
    def get(self):
        parser = reqparse.RequestParser()
        parser.add_argument("operation", type=parameters.valid_operation)
        parser.add_argument("asset_id", type=parameters.valid_txid, required=True)
        parser.add_argument("last_tx", type=parameters.valid_bool, required=False)
        args = parser.parse_args()
        with current_app.config["bigchain_pool"]() as bigchain:
            txs = bigchain.get_transactions_filtered(**args)
        return [tx.to_dict() for tx in txs]

    def post(self):
        """API endpoint to push transactions to the Federation.

        Return:
            A ``dict`` containing the data about the transaction.
        """
        parser = reqparse.RequestParser()
        parser.add_argument(
            "mode", type=parameters.valid_mode, default=BROADCAST_TX_ASYNC
        )
        args = parser.parse_args()
        mode = str(args["mode"])
        pool = current_app.config["bigchain_pool"]
        tx = request.get_json(force=True)

        log_metric(
            "received_tx",
            tx["metadata"]["requestCreationTimestamp"],
            tx["operation"],
            tx["id"],
            None
        )

        error, tx, tx_obj = validate_schema_definition(tx)
        if error is not None:
            return error
        if tx_obj.operation == Transaction.RETURN:
            return make_error(
                400, "Invalid transaction type ({})".format(tx_obj.operation)
            )

        with pool() as bigchain:
            try:
                bigchain.validate_transaction(tx_obj)
            except ValidationError as e:
                return make_error(
                    400, "Invalid transaction ({}): {}".format(type(e).__name__, e)
                )
            else:
                log_metric(
                    "before_tendermint",
                    tx_obj.metadata["requestCreationTimestamp"],
                    tx_obj.operation,
                    tx_obj._id,
                    None
                )
                status_code, message = bigchain.write_transaction(tx_obj, mode)

        if status_code == 202:
            response = jsonify(tx)
            response.status_code = 202
            return response
        else:
            return make_error(status_code, message)


class BidsForRFQTransactionApi(Resource):
    def get(self, tx_id):
        """API endpoint to get all bids raised against a RFQ transaction.

        Args:
            tx_id (str): the id of the RFQ transaction.

        Return:
            A JSON string containing the data about the Bid transactions.
        """
        pool = current_app.config["bigchain_pool"]

        with pool() as bigchain:
            txs = bigchain.get_locked_bid_txids_for_rfq(tx_id)

        return txs
