<?php

// A small mysqli CRUD demo against a live MySQL / MariaDB server.
//
// Configuration comes from the environment — either the PDO-style DSN used by
// the test suite:
//   ELEPHC_MY_DSN='mysql:host=127.0.0.1;port=3306;dbname=testdb;user=test;password=test'
// or the individual variables:
//   ELEPHC_MY_HOST, ELEPHC_MY_USER, ELEPHC_MY_PASSWORD, ELEPHC_MY_DB (and
//   optionally ELEPHC_MY_PORT).
//
// With neither set, the demo prints a note and exits cleanly, so compiling and
// running it never requires a database.

mysqli_report(MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT);

$host = (string) getenv("ELEPHC_MY_HOST");
$user = (string) getenv("ELEPHC_MY_USER");
$pass = (string) getenv("ELEPHC_MY_PASSWORD");
$dbname = (string) getenv("ELEPHC_MY_DB");
$port = 3306;
$portEnv = (string) getenv("ELEPHC_MY_PORT");
if ($portEnv !== "") {
    $port = (int) $portEnv;
}

// Fall back to parsing the PDO-style mysql: DSN the elephc test suite uses.
$dsn = (string) getenv("ELEPHC_MY_DSN");
if ($host === "" && str_starts_with($dsn, "mysql:")) {
    foreach (explode(";", substr($dsn, 6)) as $pair) {
        $eq = strpos($pair, "=");
        if ($eq === false) {
            continue;
        }
        $key = substr($pair, 0, $eq);
        $value = substr($pair, $eq + 1);
        if ($key === "host") {
            $host = $value;
        } elseif ($key === "port") {
            $port = (int) $value;
        } elseif ($key === "dbname") {
            $dbname = $value;
        } elseif ($key === "user") {
            $user = $value;
        } elseif ($key === "password") {
            $pass = $value;
        }
    }
}

if ($host === "") {
    echo "mysqli-crud: set ELEPHC_MY_DSN or ELEPHC_MY_HOST/USER/PASSWORD/DB to run the demo.\n";
    exit(0);
}

$db = new mysqli($host, $user, $pass, $dbname, $port);
echo "Connected to MySQL ", $db->server_info, " (", $db->host_info, ")\n";

// Create.
$db->query("DROP TABLE IF EXISTS mysqli_crud_demo");
$db->query("CREATE TABLE mysqli_crud_demo (
    id INT PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(64) NOT NULL,
    score DOUBLE NOT NULL
)");

// Insert through a prepared statement, re-binding per row.
$ins = $db->prepare("INSERT INTO mysqli_crud_demo (name, score) VALUES (?, ?)");
$ins->execute(["Ada", "92.5"]);
$ins->execute(["Grace", "88.0"]);
$name = "Edsger";
$score = 95.25;
$ins->bind_param("sd", $name, $score);
$ins->execute();
echo "Inserted 3 rows, last id ", $ins->insert_id > 0 ? "assigned" : "missing", "\n";
$ins->close();

// Update with an escaped ad-hoc statement.
$who = $db->real_escape_string("Grace");
$db->query("UPDATE mysqli_crud_demo SET score = score + 1 WHERE name = '" . $who . "'");
echo "Updated ", $db->affected_rows, " row(s)\n";

// Read back: buffered results stay independent of later queries.
$rows = $db->query("SELECT id, name, score FROM mysqli_crud_demo ORDER BY score DESC");
$count = $db->query("SELECT COUNT(*) AS c FROM mysqli_crud_demo");
if ($rows instanceof mysqli_result && $count instanceof mysqli_result) {
    $total = $count->fetch_assoc();
    echo $total["c"], " rows:\n";
    foreach ($rows as $row) {
        echo "  #", $row["id"], " ", $row["name"], " -> ", $row["score"], "\n";
    }
}

// Delete inside a transaction.
$db->begin_transaction();
$db->query("DELETE FROM mysqli_crud_demo WHERE score < 90");
$db->commit();
echo "Deleted low scores, ", $db->affected_rows, " row(s)\n";

// Drop the demo table and close.
$db->query("DROP TABLE mysqli_crud_demo");
$db->close();
echo "Done.\n";
