import time

from bigchaindb.lib import BigchainDB
from bigchaindb.common.transaction_mode_types import (
    BROADCAST_TX_ASYNC,
    BROADCAST_TX_SYNC,
    BROADCAST_TX_COMMIT,
)


class ReturnExecutor(object):
    def __execute(tx):
        bigchain = BigchainDB()
        status_code, _ = bigchain.write_transaction(tx, BROADCAST_TX_SYNC)
        # while status_code != 202:
        #     status_code, _ = bigchain.write_transaction(tx, BROADCAST_TX_SYNC)

    @classmethod
    def worker(cls, pool, return_queue):
        timeout = 1
        while True:
            if not return_queue.empty():
                timeout = 1
                return_tx = return_queue.get()
                _future = pool.submit(cls.__execute, return_tx)
            else:
                # timeout = timeout * 2
                timeout = 5

            time.sleep(timeout)
