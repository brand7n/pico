<?php

declare(strict_types=1);

/**
 * Test: Pico runtime under standard PHP.
 *
 * Demonstrates that Collection bracket syntax, PicoString bracket access,
 * Collection::from(), and @template type tracking all work as real PHP.
 */

require_once __DIR__ . '/src/PicoString.php';
require_once __DIR__ . '/src/Collection.php';
require_once __DIR__ . '/src/StringBuilder.php';
require_once __DIR__ . '/src/File.php';

use Remodulate\Pico\{Collection, PicoString, StringBuilder};

$passed = 0;
$failed = 0;

function check(string $name, bool $condition): void
{
    global $passed, $failed;
    if ($condition) {
        echo "  \033[32m✓\033[0m {$name}\n";
        $passed++;
    } else {
        echo "  \033[31m✗\033[0m {$name}\n";
        $failed++;
    }
}

echo "═══ Collection: Sequential access ═══\n\n";

/** @var Collection<int> */
$ids = Collection::from([10, 20, 30]);

check('from() creates collection with 3 elements', $ids->count() === 3);
check('bracket read $ids[0]', $ids[0] === 10);
check('bracket read $ids[2]', $ids[2] === 30);
check('getAt() same as bracket', $ids->getAt(1) === $ids[1]);

$ids[] = 40;
check('bracket push $ids[] = 40', $ids->count() === 4);
check('pushed value accessible', $ids[3] === 40);

$ids[1] = 99;
check('bracket write $ids[1] = 99', $ids[1] === 99);

$last = $ids->pop();
check('pop() returns last', $last === 40);
check('pop() reduces count', $ids->count() === 3);

check('last() returns last without removing', $ids->last() === 30);
check('validIndex(0)', $ids->validIndex(0));
check('!validIndex(99)', !$ids->validIndex(99));

echo "\n═══ Collection: Associative access ═══\n\n";

/** @var Collection<int> */
$map = new Collection();
$map['width'] = 1920;
$map['height'] = 1080;

check('string key set + get', $map['width'] === 1920);
check('has() for existing key', $map->has('width'));
check('!has() for missing key', !$map->has('depth'));
check('count() with string keys', $map->count() === 2);
check('keyAt(0)', $map->keyAt(0) === 'width');

echo "\n═══ Collection: Mixed key types ═══\n\n";

/** @var Collection<string> */
$mixed = new Collection();
$mixed[] = 'first';
$mixed[] = 'second';
$mixed['name'] = 'hello';

check('sequential + associative coexist', $mixed->count() === 3);
check('int key access', $mixed[0] === 'first');
check('string key access', $mixed['name'] === 'hello');

echo "\n═══ Collection: Search and contains ═══\n\n";

/** @var Collection<int> */
$nums = Collection::from([5, 10, 15, 20]);

check('indexOf found', $nums->indexOf(15) === 2);
check('indexOf not found', $nums->indexOf(99) === -1);
check('containsValue true', $nums->containsValue(10));
check('containsValue false', !$nums->containsValue(99));

echo "\n═══ Collection: Slice ═══\n\n";

$sliced = $nums->slice(1, 2);
check('slice count', $sliced->count() === 2);
check('slice values', $sliced[0] === 10 && $sliced[1] === 15);

echo "\n═══ Collection: isset() via ArrayAccess ═══\n\n";

check('isset($nums[0])', isset($nums[0]));
check('!isset($nums[99])', !isset($nums[99]));

echo "\n═══ Collection: of() factory ═══\n\n";

$fromOf = Collection::of('a', 'b', 'c');
check('of() creates 3 elements', $fromOf->count() === 3);
check('of() element access', $fromOf[0] === 'a' && $fromOf[2] === 'c');

echo "\n═══ PicoString: Basic operations ═══\n\n";

$s = new PicoString("Hello, World!");

check('length()', $s->length() === 13);
check('charAt(0)', $s->charAt(0) === ord('H'));
check('bracket access $s[0]', $s[0] === ord('H'));
check('bracket access $s[7]', $s[7] === ord('W'));
check('substring(0, 5)', $s->substring(0, 5)->equals(new PicoString('Hello')));
check('substring(7)', $s->substring(7)->equals(new PicoString('World!')));

$needle = new PicoString('World');
check('indexOf()', $s->indexOf($needle) === 7);
check('indexOf() not found', $s->indexOf(new PicoString('xyz')) === -1);
check('startsWith()', $s->startsWith(new PicoString('Hello')));
check('!startsWith()', !$s->startsWith(new PicoString('World')));
check('endsWith()', $s->endsWith(new PicoString('World!')));
check('contains()', $s->contains($needle));
check('!contains()', !$s->contains(new PicoString('xyz')));

check('concat()', $s->concat(new PicoString('!'))->equals(new PicoString('Hello, World!!')));
check('trim()', (new PicoString("  hi  "))->trim()->equals(new PicoString('hi')));
check('toLower()', $s->toLower()->equals(new PicoString('hello, world!')));
check('toUpper()', $s->toUpper()->equals(new PicoString('HELLO, WORLD!')));

$replaced = $s->replace(new PicoString('World'), new PicoString('Pico'));
check('replace()', $replaced->equals(new PicoString('Hello, Pico!')));

check('toInt()', (new PicoString('42'))->toInt() === 42);

echo "\n═══ PicoString: Split ═══\n\n";

$csv = new PicoString("a,b,c");
$parts = $csv->split(new PicoString(','));
check('split count', $parts->count() === 3);
check('split values', $parts[0]->equals(new PicoString('a')));
check('split values', $parts[2]->equals(new PicoString('c')));

echo "\n═══ PicoString: Immutability ═══\n\n";

$caught = false;
try {
    $s[0] = 65;
} catch (\RuntimeException $e) {
    $caught = true;
}
check('bracket write throws on PicoString', $caught);

echo "\n═══ StringBuilder ═══\n\n";

$sb = new StringBuilder();
$sb->append(new PicoString('define i32 @main() {'));
$sb->appendChar(10); // newline
$sb->append(new PicoString('    ret i32 '));
$sb->appendInt(42);
$sb->appendChar(10);
$sb->append(new PicoString('}'));

$output = $sb->toString();
check('builder length', $sb->length() === 37);
check('builder contains ret', $output->contains(new PicoString('ret i32 42')));
check('builder starts with define', $output->startsWith(new PicoString('define')));

$sb->clear();
check('clear resets length', $sb->length() === 0);

echo "\n═══ Realistic: Token stream ═══\n\n";

// Simulates how the compiler would use these classes
class Token
{
    public function __construct(
        public readonly int $kind,
        public readonly PicoString $value,
        public readonly int $line,
    ) {}
}

/** @var Collection<Token> */
$tokens = new Collection();
$tokens[] = new Token(268, new PicoString('function'), 1);
$tokens[] = new Token(307, new PicoString('main'), 1);
$tokens[] = new Token(40, new PicoString('('), 1);     // ASCII '('
$tokens[] = new Token(41, new PicoString(')'), 1);     // ASCII ')'

check('token stream count', $tokens->count() === 4);
check('token[0] kind', $tokens[0]->kind === 268);
check('token[0] value', $tokens[0]->value->equals(new PicoString('function')));
check('token[1] value', $tokens[1]->value->equals(new PicoString('main')));

// Find a token by value
$found = -1;
for ($i = 0; $i < $tokens->count(); $i++) {
    if ($tokens[$i]->value->equals(new PicoString('main'))) {
        $found = $i;
        break;
    }
}
check('search token by value', $found === 1);

echo "\n═══ Realistic: Symbol table ═══\n\n";

class SymbolInfo
{
    public function __construct(
        public readonly PicoString $name,
        public readonly PicoString $type,
        public readonly int $scopeDepth,
    ) {}
}

/** @var Collection<SymbolInfo> */
$symbols = new Collection();
$symbols['x'] = new SymbolInfo(new PicoString('x'), new PicoString('int'), 0);
$symbols['y'] = new SymbolInfo(new PicoString('y'), new PicoString('float'), 0);
$symbols['point'] = new SymbolInfo(new PicoString('point'), new PicoString('Point'), 1);

check('symbol table count', $symbols->count() === 3);
check('lookup by name', $symbols['x']->type->equals(new PicoString('int')));
check('has symbol', $symbols->has('point'));
check('!has symbol', !$symbols->has('z'));
check('symbol scope', $symbols['point']->scopeDepth === 1);

// ═══ Results ═══

echo "\n" . str_repeat('═', 50) . "\n";
echo "Results: \033[32m{$passed} passed\033[0m";
if ($failed > 0) {
    echo ", \033[31m{$failed} failed\033[0m";
}
echo " out of " . ($passed + $failed) . " tests\n";
exit($failed > 0 ? 1 : 0);
