<?php

final class Address
{
    public string $city = 'Unknown';
}

final class Customer
{
    public Address $address;
}

$customer = new Customer();
$customer->address = new Address();
$customer->address->city = 'Paris';
echo $customer->address->city, "\n";
