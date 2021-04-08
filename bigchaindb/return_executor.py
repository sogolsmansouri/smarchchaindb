import time

from bigchaindb.lib import BigchainDB
from bigchaindb.common.transaction_mode_types import (
    BROADCAST_TX_ASYNC,
    BROADCAST_TX_SYNC,
    BROADCAST_TX_COMMIT,
)


class ReturnExecutor(object):
    bigchain = BigchainDB()

    def __execute(tx):
        status_code, _ = ReturnExecutor.bigchain.write_transaction(
            tx, BROADCAST_TX_SYNC
        )
        # while status_code != 202:
        #     status_code, _ = bigchain.write_transaction(tx, BROADCAST_TX_SYNC)

    @classmethod
    def worker(cls, pool, return_queue):
        timeout = 5
        while True:
            if not return_queue.empty():
                return_tx = return_queue.get()
                _future = pool.submit(cls.__execute, return_tx)
            else:
                time.sleep(timeout)
