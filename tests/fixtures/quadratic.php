<?php

foreach ($users as $user) {
    foreach ($orders as $order) {
        if (in_array($order->id, $user->order_ids, true)) {
            echo $order->id;
        }
    }
}
