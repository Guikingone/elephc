<?php

namespace App\Controller;

use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\JsonResponse;
use Symfony\Component\HttpFoundation\Request;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\Routing\Attribute\Route;

/**
 * The first application code this example runs under a compiled kernel.
 *
 * Until it existed the app had no controller at all, so every request 404ed and the only response
 * elephc had ever produced was the prod error page. Each action here exercises one thing the 404
 * path never reached: argument resolution from the route, reading the request, the two response
 * shapes an app actually returns, and — with `/` — Twig, which compiles each template to a PHP
 * class at runtime and `require`s it, so the whole render travels the interpreted-include path.
 */
final class HelloController extends AbstractController
{
    #[Route('/', name: 'home', methods: ['GET'])]
    public function home(): Response
    {
        return $this->render('hello.html.twig', [
            'greeting' => 'Hello from Symfony',
            'engine' => 'Twig',
            'words' => ['compiled', 'ahead', 'of', 'time'],
        ]);
    }

    #[Route('/plain', name: 'plain', methods: ['GET'])]
    public function plain(): Response
    {
        return new Response("hello from a compiled controller\n");
    }

    #[Route('/greet/{name}', name: 'greet', methods: ['GET'])]
    public function greet(string $name): Response
    {
        return new Response(sprintf("hello %s\n", strtoupper($name)));
    }

    #[Route('/echo', name: 'echo', methods: ['GET', 'POST'])]
    public function echoRequest(Request $request): JsonResponse
    {
        return new JsonResponse([
            'method' => $request->getMethod(),
            'query' => $request->query->all(),
            'post' => $request->request->all(),
            'path' => $request->getPathInfo(),
        ]);
    }
}
