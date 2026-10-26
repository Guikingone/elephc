<?php

final class Address
{
    public string $city = 'Unknown';
}

final class Customer
{
    public Address $billing;
    public Address $shipping;
}

$address = new Address();
$address->city = 'Paris';
$customer = new Customer();
$customer->billing = $address;
$customer->shipping = $address;
$copy = unserialize(serialize($customer));
echo $copy->billing->city, ' / ',
    $copy->billing === $copy->shipping ? 'shared address' : 'separate addresses', "\n";
