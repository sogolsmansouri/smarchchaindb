# Copyright © 2020 Interplanetary Database Association e.V.,
# BigchainDB and IPDB software contributors.
# SPDX-License-Identifier: (Apache-2.0 AND CC-BY-4.0)
# Code is Apache-2.0 and docs are CC-BY-4.0

"""Schema validation related functions and data"""
import os.path
import logging

import jsonschema
import yaml
import rapidjson

from bigchaindb.common.exceptions import SchemaValidationError


logger = logging.getLogger(__name__)


def _load_schema(name, path=__file__):
    """Load a schema from disk"""
    path = os.path.join(os.path.dirname(path), name + ".yaml")
    with open(path) as handle:
        schema = yaml.safe_load(handle)
    fast_schema = rapidjson.Validator(rapidjson.dumps(schema))
    return path, (schema, fast_schema)


TX_SCHEMA_VERSION = "v2.0"

TX_SCHEMA_PATH, TX_SCHEMA_COMMON = _load_schema("transaction_" + TX_SCHEMA_VERSION)
_, TX_SCHEMA_CREATE = _load_schema("transaction_create_" + TX_SCHEMA_VERSION)
_, TX_SCHEMA_TRANSFER = _load_schema("transaction_transfer_" + TX_SCHEMA_VERSION)

_, TX_SCHEMA_VALIDATOR_ELECTION = _load_schema(
    "transaction_validator_election_" + TX_SCHEMA_VERSION
)

_, TX_SCHEMA_CHAIN_MIGRATION_ELECTION = _load_schema(
    "transaction_chain_migration_election_" + TX_SCHEMA_VERSION
)

_, TX_SCHEMA_VOTE = _load_schema("transaction_vote_" + TX_SCHEMA_VERSION)

_, TX_SCHEMA_PRE_REQUEST = _load_schema("transaction_update_adv_" + TX_SCHEMA_VERSION)

_, TX_SCHEMA_INTEREST = _load_schema("transaction_accept_return_" + TX_SCHEMA_VERSION)

_, TX_SCHEMA_REQUEST_FOR_QUOTE = _load_schema(
    "transaction_request_for_quote_" + TX_SCHEMA_VERSION
)

_, TX_SCHEMA_BID = _load_schema("transaction_bid_" + TX_SCHEMA_VERSION)

_, TX_SCHEMA_ACCEPT = _load_schema("transaction_accept_" + TX_SCHEMA_VERSION)

_, TX_SCHEMA_RETURN = _load_schema("transaction_return_" + TX_SCHEMA_VERSION)

_, TX_SCHEMA_ADV = _load_schema("transaction_adv_" + TX_SCHEMA_VERSION)
_, TX_SCHEMA_BUY = _load_schema("transaction_buy_offer_" + TX_SCHEMA_VERSION)
_, TX_SCHEMA_SELL = _load_schema("transaction_sell_" + TX_SCHEMA_VERSION)
_, TX_SCHEMA_INVERSE_TXN = _load_schema("transaction_inverse_txn_" + TX_SCHEMA_VERSION)
_, TX_SCHEMA_ACCEPT_RETURN = _load_schema("transaction_accept_return_" + TX_SCHEMA_VERSION)
_, TX_SCHEMA_UPDATE_ADV = _load_schema("transaction_update_adv_" + TX_SCHEMA_VERSION)


def _validate_schema(schema, body):
    """Validate data against a schema"""

    # Note
    #
    # Schema validation is currently the major CPU bottleneck of
    # BigchainDB. the `jsonschema` library validates python data structures
    # directly and produces nice error messages, but validation takes 4+ ms
    # per transaction which is pretty slow. The rapidjson library validates
    # much faster at 1.5ms, however it produces _very_ poor error messages.
    # For this reason we use both, rapidjson as an optimistic pathway and
    # jsonschema as a fallback in case there is a failure, so we can produce
    # a helpful error message.

    try:
        schema[1](rapidjson.dumps(body))
    except ValueError as exc:
        try:
            jsonschema.validate(body, schema[0])
        except jsonschema.ValidationError as exc2:
            raise SchemaValidationError(str(exc2)) from exc2
        logger.warning(
            "code problem: jsonschema did not raise an exception, wheras rapidjson raised %s",
            exc,
        )
        raise SchemaValidationError(str(exc)) from exc


def validate_transaction_schema(tx):
    """Validate a transaction dict.

    TX_SCHEMA_COMMON contains properties that are common to all types of
    transaction. TX_SCHEMA_[TRANSFER|CREATE|REQUEST_FOR_QUOTE|INTEREST|ACCEPT] add additional constraints on top.
    """
    _validate_schema(TX_SCHEMA_COMMON, tx)
    if tx["operation"] == "TRANSFER":
        _validate_schema(TX_SCHEMA_TRANSFER, tx)
    elif tx["operation"] == "PRE_REQUEST":
        _validate_schema(TX_SCHEMA_INVERSE_TXN, tx)
    elif tx["operation"] == "INTEREST":
        _validate_schema(TX_SCHEMA_INTEREST, tx)
    elif tx["operation"] == "REQUEST_FOR_QUOTE":
        _validate_schema(TX_SCHEMA_UPDATE_ADV, tx)
    elif tx["operation"] == "BID":
        _validate_schema(TX_SCHEMA_BID, tx)
    elif tx["operation"] == "ACCEPT":
        _validate_schema(TX_SCHEMA_ACCEPT, tx)
    elif tx["operation"] == "INVERSE_TXN":
        _validate_schema(TX_SCHEMA_INVERSE_TXN, tx)
    elif tx["operation"] == "RETURN":
        _validate_schema(TX_SCHEMA_RETURN, tx)
    elif tx["operation"] == "BUYOFFER":
        _validate_schema(TX_SCHEMA_BUY, tx)
    elif tx["operation"] == "ADV":
        _validate_schema(TX_SCHEMA_ADV, tx)
    elif tx["operation"] == "SELL":
        _validate_schema(TX_SCHEMA_SELL, tx)
    elif tx["operation"] == "ACCEPT_RETURN":
        _validate_schema(TX_SCHEMA_ACCEPT_RETURN, tx)
    elif tx["operation"] == "UPDATE_ADV":
        _validate_schema(TX_SCHEMA_UPDATE_ADV, tx)
        
    else:
        _validate_schema(TX_SCHEMA_CREATE, tx)