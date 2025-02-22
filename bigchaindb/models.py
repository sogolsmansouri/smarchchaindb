# Copyright © 2020 Interplanetary Database Association e.V.,
# BigchainDB and IPDB software contributors.
# SPDX-License-Identifier: (Apache-2.0 AND CC-BY-4.0)
# Code is Apache-2.0 and docs are CC-BY-4.0

from bigchaindb.backend.schema import validate_language_key
from bigchaindb.common.exceptions import InvalidSignature, DuplicateTransaction
from bigchaindb.common.schema import validate_transaction_schema
from bigchaindb.common.transaction import Transaction
from bigchaindb.common.utils import validate_txn_obj, validate_key


class Transaction(Transaction):
    ASSET = "asset"
    METADATA = "metadata"
    DATA = "data"

    def validate(self, bigchain, current_transactions=[]):
        
        """Validate transaction spend
        Args:
            bigchain (BigchainDB): an instantiated bigchaindb.BigchainDB object.
        Returns:
            The transaction (Transaction) if the transaction is valid else it
            raises an exception describing the reason why the transaction is
            invalid.
        Raises:
            ValidationError: If the transaction is invalid
        """
        input_conditions = []
        duplicates = any(txn for txn in current_transactions if txn.id == self.id)
        if bigchain.is_committed(self.id) or duplicates:
            raise DuplicateTransaction(
                "transaction `{}` already exists".format(self.id)
            )

        if (
            self.operation
            in [
                Transaction.CREATE,
                #Transaction.PRE_REQUEST,
                #Transaction.INTEREST,
                Transaction.REQUEST_FOR_QUOTE,
                Transaction.ACCEPT,
                Transaction.ADV,
                Transaction.UPDATE_ADV,
            ]
            and not self.inputs_valid(input_conditions, bigchain)
        ):
            raise InvalidSignature("Transaction signature is invalid.")
        
        if self.operation == Transaction.TRANSFER:
            self.validate_transfer_inputs(bigchain, current_transactions)
        elif self.operation == Transaction.INTEREST:
            self.validate_accept_return(bigchain, current_transactions)
        elif self.operation == Transaction.REQUEST_FOR_QUOTE:
            self.validate_update_adv(bigchain, current_transactions)
        elif self.operation == Transaction.BID:
            self.validate_bid(bigchain, current_transactions)
        elif self.operation == Transaction.ACCEPT:
            self.validate_update_adv(bigchain, current_transactions)
        elif self.operation == Transaction.RETURN:
            self.validate_return(bigchain, current_transactions)
        elif self.operation == Transaction.BUYOFFER:
            self.validate_buy_offer(bigchain, current_transactions)
        elif self.operation == Transaction.ADV:
            self.validate_adv(bigchain, current_transactions)
        elif self.operation == Transaction.SELL:
            self.validate_sell(bigchain, current_transactions)
        elif self.operation == Transaction.PRE_REQUEST:
            self.validate_inverse_txn(bigchain, current_transactions)
        elif self.operation == Transaction.ACCEPT_RETURN:
            self.validate_accept_return(bigchain, current_transactions)

        return self

    @classmethod
    def from_dict(cls, tx_body):
        return super().from_dict(tx_body, False)

    @classmethod
    def validate_schema(cls, tx_body):
        validate_transaction_schema(tx_body)
        validate_txn_obj(cls.ASSET, tx_body[cls.ASSET], cls.DATA, validate_key)
        validate_txn_obj(cls.METADATA, tx_body, cls.METADATA, validate_key)
        validate_language_key(tx_body[cls.ASSET], cls.DATA)
        validate_language_key(tx_body, cls.METADATA)


class FastTransaction:
    """A minimal wrapper around a transaction dictionary. This is useful for
    when validation is not required but a routine expects something that looks
    like a transaction, for example during block creation.

    Note: immutability could also be provided
    """

    def __init__(self, tx_dict):
        self.data = tx_dict

    @property
    def id(self):
        return self.data["id"]

    def to_dict(self):
        return self.data